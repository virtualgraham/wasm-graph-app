use std::rc::Rc;
use std::cell::RefCell;
use io_context::Context;
use super::super::graph::iterator;
use super::super::graph::hasa::HasA;
use super::super::graph::value::Value;
use super::super::graph::linksto::LinksTo;
use super::super::graph::refs::{Ref, Content};
use super::super::graph::quad::{QuadStore, Direction};


pub enum ShapeType {
    Lookup,
    Null,
    Fixed(Fixed),
    AllNodes,
    Intersect,
    NodesFrom,
    QuadFilter,
    Quads,
    Save,
    Union
}


pub trait Shape {
    fn build_iterator(&self, qs: Rc<RefCell<dyn QuadStore>>) -> Rc<RefCell<dyn iterator::Shape>>;
    fn optimize(&mut self, ctx: &Context, o: Option<&dyn Optimizer>) -> Option<Rc<RefCell<dyn Shape>>>;
    fn shape_type(self) -> ShapeType;
}

pub trait Optimizer{
    fn optimize_shape(&self, ctx: &Context, shape: &mut dyn Shape) -> Option<Rc<RefCell<dyn Shape>>>;
    fn quad_store(&self) -> Option<Rc<RefCell<dyn QuadStore>>>;
}

pub trait Composite {
    fn simplify(&self) -> Rc<RefCell<dyn Shape>>;
}

pub trait WalkFunc {
    fn walk(&self, shape: Rc<RefCell<dyn Shape>>) -> bool;
}


struct ResolveValues {
    qs: Rc<RefCell<dyn QuadStore>>
}

// impl<T: QuadStore> Optimizer for ResolveValues<T> {
//     fn optimize_shape(&self, shape: Rc<RefCell<dyn iterator::Shape>>) -> Option<Rc<RefCell<dyn iterator::Shape>>> {
//         if let iterator::ShapeType::Lookup = shape.borrow().shape_type() {
            
//         }
//         return None
//     }
// }


///////////////////////////////////////////////


pub struct Lookup (pub Vec<Value>);


impl Lookup {
    pub fn new(values: Vec<Value>) -> Rc<RefCell<Lookup>> {
        Rc::new(RefCell::new(Lookup(values)))
    }

    fn add(&mut self, values: Vec<Value>) {
        self.0.extend(values);
    }

    fn resolve(&self, qs: Rc<RefCell<dyn QuadStore>>) -> Option<Rc<RefCell<dyn Shape>>> {
        let mut vals:Vec<Ref> = Vec::new();
        for v in &self.0 {
            let gv = qs.borrow().value_of(v);
            if gv.is_some() {
                vals.push(gv.unwrap());
            }
        }
        if vals.is_empty() {
            return None
        }
        return Some(Fixed::new(vals))
    }
}

impl Shape for Lookup {
    fn build_iterator(&self, qs: Rc<RefCell<dyn QuadStore>>) -> Rc<RefCell<dyn iterator::Shape>> {
        let f = self.resolve(qs.clone());
        if f.is_none() {
            return iterator::Null::new();
        }
        return f.unwrap().borrow().build_iterator(qs)
    }

    fn optimize(&mut self, ctx: &Context, o: Option<&dyn Optimizer>) -> Option<Rc<RefCell<dyn Shape>>> {
        if o.is_none() {
            return None
        }
        let ns = o.unwrap().optimize_shape(ctx, self);
        if ns.is_some() {
            return ns
        }

        if o.unwrap().quad_store().is_some() {
            let optimizer = o.unwrap();
            let qs_rc = optimizer.quad_store().unwrap();
            return self.resolve(qs_rc)
        }

        return None
    }

    fn shape_type(self) -> ShapeType {
        ShapeType::Lookup
    }
}


///////////////////////////////////////////////


pub struct Fixed (pub Vec<Ref>);

impl Fixed {
    pub fn new(refs: Vec<Ref>) -> Rc<RefCell<dyn Shape>> {
        Rc::new(RefCell::new(Fixed(refs)))
    }
}

impl Shape for Fixed {
    fn build_iterator(&self, qs: Rc<RefCell<dyn QuadStore>>) -> Rc<RefCell<dyn iterator::Shape>> {
        let it = iterator::fixed::Fixed::new(vec![]);
        for v in &self.0 {
            if let Content::Quad(_) = v.content {
                panic!("quad value in fixed iterator")
            }
            it.borrow_mut().add(v.clone());
        }
        return it;
    }

    fn optimize(&mut self, ctx: &Context, r: Option<&dyn Optimizer>) -> Option<Rc<RefCell<dyn Shape>>> {
        if self.0.is_empty() {
            return None
        }
        if let Some(o) = r {
            return o.optimize_shape(ctx, self)
        }
        return None
    }

    fn shape_type(self) -> ShapeType {
        ShapeType::Fixed(self)
    }
}


///////////////////////////////////////////////


pub struct Null ();

impl Null {
    pub fn new() -> Rc<RefCell<Null>> {
        Rc::new(RefCell::new(Null()))
    }
}

impl Shape for Null {
    fn build_iterator(&self, qs: Rc<RefCell<dyn QuadStore>>) -> Rc<RefCell<dyn iterator::Shape>> {
        return iterator::Null::new();
    }

    fn optimize(&mut self, ctx: &Context, r: Option<&dyn Optimizer>) -> Option<Rc<RefCell<dyn Shape>>>  {
        if r.is_some() {
            return r.unwrap().optimize_shape(ctx, self)
        }
        return None
    }

    fn shape_type(self) -> ShapeType {
        ShapeType::Null
    }
}


///////////////////////////////////////////////


pub struct AllNodes ();

impl AllNodes {
    pub fn new() -> Rc<RefCell<AllNodes>> {
        Rc::new(RefCell::new(AllNodes()))
    }
}


impl Shape for AllNodes {
    fn build_iterator(&self, qs: Rc<RefCell<dyn QuadStore>>) -> Rc<RefCell<dyn iterator::Shape>> {
        qs.borrow().nodes_all_iterator()
    }

    fn optimize(&mut self, ctx: &Context, r: Option<&dyn Optimizer>) -> Option<Rc<RefCell<dyn Shape>>>  {
        if r.is_some() {
            return r.unwrap().optimize_shape(ctx, self)
        }
        return None      
    }

    fn shape_type(self) -> ShapeType {
        ShapeType::AllNodes
    }
}


///////////////////////////////////////////////


pub struct Intersect (pub Vec<Rc<RefCell<dyn Shape>>>);

impl Intersect {
    pub fn new(values: Vec<Rc<RefCell<dyn Shape>>>) -> Rc<RefCell<dyn Shape>> {
        Rc::new(RefCell::new(Intersect(values)))
    }
}


impl Shape for Intersect {
    fn build_iterator(&self, qs: Rc<RefCell<dyn QuadStore>>) -> Rc<RefCell<dyn iterator::Shape>> {
        if self.0.is_empty() {
            return iterator::Null::new()
        }
        let mut sub = Vec::new();
        for c in &self.0 {
            sub.push(c.borrow().build_iterator(qs.clone()));
        }
        if sub.len() == 1 {
            return sub[0].clone()
        }
        return iterator::and::And::new(sub)
    }

    fn optimize(&mut self, ctx: &Context, r: Option<&dyn Optimizer>) -> Option<Rc<RefCell<dyn Shape>>> {
        if self.0.is_empty() {
            return None
        }

        for i in 0..self.0.len() {
            let c = &self.0[i];
            if let ShapeType::Null = c.borrow().shape_type() {
                return None
            }
            let v = c.borrow_mut().optimize(ctx, r);
            if v.is_none() {
                continue;
            }
            if let ShapeType::Null = v.as_ref().unwrap().borrow().shape_type() {
                return None
            }
            self.0[i] = v.unwrap();
        }

        return None
    }

    fn shape_type(self) -> ShapeType {
        ShapeType::Intersect
    }
}


///////////////////////////////////////////////


pub struct NodesFrom {
    dir: Direction,
    quads: Rc<RefCell<dyn Shape>>
}

impl Shape for NodesFrom {
    fn build_iterator(&self, qs: Rc<RefCell<dyn QuadStore>>) -> Rc<RefCell<dyn iterator::Shape>> {
        if let ShapeType::Null = self.quads.borrow().shape_type() {
            return iterator::Null::new() 
        }
        let sub = self.quads.borrow().build_iterator(qs);
        if let Direction::Any = self.dir {
            panic!("direction is not set");
        }
        return HasA::new(qs, sub, self.dir)
    }

    fn optimize(&mut self, ctx: &Context, r: Option<&dyn Optimizer>) -> Option<Rc<RefCell<dyn Shape>>> {
        return None
    }

    fn shape_type(self) -> ShapeType {
        ShapeType::NodesFrom
    }
}


///////////////////////////////////////////////


pub struct QuadFilter {
    dir: Direction,
    values: Option<Rc<RefCell<dyn Shape>>>
}

impl Shape for QuadFilter {
    fn build_iterator(&self, qs: Rc<RefCell<dyn QuadStore>>) -> Rc<RefCell<dyn iterator::Shape>> {
        if self.values.is_none() {
            return iterator::Null::new() 
        }

        if let Some(v) = one(self.values.unwrap().clone()) {
            return qs.borrow().quad_iterator(&self.dir, &v)
        }

        if let Direction::Any = self.dir {
            panic!("direction is not set")
        }

        let sub = self.values.unwrap().borrow().build_iterator(qs.clone());

        LinksTo::new(qs.clone(), sub, self.dir.clone())
    }

    fn optimize(&mut self, ctx: &Context, r: Option<&dyn Optimizer>) -> Option<Rc<RefCell<dyn Shape>>> {
        return None
    }

    fn shape_type(self) -> ShapeType {
        ShapeType::QuadFilter
    }
}


///////////////////////////////////////////////


pub struct Quads(pub Vec<QuadFilter>);

impl Quads {
    fn interset(&mut self, q: Vec<QuadFilter>) {
        self.0.append(&mut q)
    }
}

impl Shape for Quads {
    fn build_iterator(&self, qs: Rc<RefCell<dyn QuadStore>>) -> Rc<RefCell<dyn iterator::Shape>> {

    }

    fn optimize(&mut self, ctx: &Context, r: Option<&dyn Optimizer>) -> Option<Rc<RefCell<dyn Shape>>> {
        return None
    }

    fn shape_type(self) -> ShapeType {
        ShapeType::Quads
    }
}


///////////////////////////////////////////////

pub struct Save {
    tags: Vec<String>,
    from: Rc<RefCell<dyn Shape>>
}

impl Shape for Save {
    fn build_iterator(&self, qs: Rc<RefCell<dyn QuadStore>>) -> Rc<RefCell<dyn iterator::Shape>> {

    }

    fn optimize(&mut self, ctx: &Context, r: Option<&dyn Optimizer>) -> Option<Rc<RefCell<dyn Shape>>> {
        return None
    }

    fn shape_type(self) -> ShapeType {
        ShapeType::Save
    }
}


///////////////////////////////////////////////

pub struct Union(pub Vec<Rc<RefCell<dyn Shape>>>);

impl Shape for Union {
    fn build_iterator(&self, qs: Rc<RefCell<dyn QuadStore>>) -> Rc<RefCell<dyn iterator::Shape>> {
        return iterator::Null::new()
    }

    fn optimize(&mut self, ctx: &Context, r: Option<&dyn Optimizer>) -> Option<Rc<RefCell<dyn Shape>>> {
        return None
    }

    fn shape_type(self) -> ShapeType {
        ShapeType::Union
    }
}







// buildOut() from query/shape/path.go
pub fn new_in_out(from:Rc<RefCell<dyn Shape>>, via:Rc<RefCell<dyn Shape>>, labels:Option<Rc<RefCell<dyn Shape>>>, tags:Vec<String>, r#in: bool) -> Rc<RefCell<dyn Shape>> {
    let start = if r#in { Direction::Subject } else { Direction::Object };
    let goal = if r#in { Direction::Object } else { Direction::Subject };

    if !tags.is_empty() {
        via = Rc::new(RefCell::new(Save {
            tags: tags,
            from: via
        }));
    }

    let quads = Rc::new(RefCell::new(Quads(Vec::new())));

    if let ShapeType::AllNodes = from.borrow().shape_type() {
        quads.borrow().0.push(QuadFilter {
            dir: start,
            values: Some(from)
        });
    }

    if let ShapeType::AllNodes = via.borrow().shape_type() {
        quads.borrow().0.push(QuadFilter {
            dir: Direction::Predicate,
            values: Some(via)
        });
    }

    if labels.is_some() {
        if let ShapeType::AllNodes = labels.unwrap().borrow().shape_type() {
            quads.borrow().0.push(QuadFilter {
                dir: Direction::Label,
                values: Some(labels.unwrap())
            });
        }
    }

    Rc::new(RefCell::new(NodesFrom {
        quads,
        dir: goal
    }))
}

pub fn new_in(from:Rc<RefCell<dyn Shape>>, via:Rc<RefCell<dyn Shape>>, labels:Option<Rc<RefCell<dyn Shape>>>, tags:Vec<String>) -> Rc<RefCell<dyn Shape>> {
    new_in_out(from, via, labels, tags, true)
}

pub fn new_out(from:Rc<RefCell<dyn Shape>>, via:Rc<RefCell<dyn Shape>>, labels:Option<Rc<RefCell<dyn Shape>>>, tags:Vec<String>) -> Rc<RefCell<dyn Shape>> {
    new_in_out(from, via, labels, tags, false)
}


fn one(shape: Rc<RefCell<dyn Shape>>) -> Option<Ref> {
    if let ShapeType::Fixed(f) = shape.borrow().shape_type() {
        if f.0.len() == 1 {
            return Some(f.0[0])
        }
    }
    return None
}
