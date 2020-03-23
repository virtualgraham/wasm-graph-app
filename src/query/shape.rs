use std::rc::Rc;
use std::cell::RefCell;
use io_context::Context;
use super::path;
use super::super::graph::iterator;
use super::super::graph::hasa::HasA;
use super::super::graph::value::Value;
use super::super::graph::linksto::LinksTo;
use super::super::graph::refs::{Ref, Content};
use super::super::graph::quad::{QuadStore, Direction};
use regex::Regex;

pub enum ShapeType<'a> {
    Lookup(&'a mut Lookup),
    Null,
    Fixed(&'a mut Fixed),
    AllNodes,
    Intersect(&'a mut Intersect),
    IntersectOpt(&'a mut IntersectOpt),
    NodesFrom,
    QuadFilter,
    Quads,
    Save,
    Union,
    Recursive,
    IteratorShape,
    Filter(&'a mut Filter)
}


pub trait Shape {
    fn build_iterator(&self, qs: Rc<RefCell<dyn QuadStore>>) -> Rc<RefCell<dyn iterator::Shape>>;
    fn optimize(&mut self, ctx: &Context, o: Option<&dyn Optimizer>) -> Option<Rc<RefCell<dyn Shape>>>;
    fn shape_type(&mut self) -> ShapeType;
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
    pub qs: Rc<RefCell<dyn QuadStore>>
}

impl Optimizer for ResolveValues {
    fn optimize_shape(&self, ctx: &Context, shape: &mut dyn Shape) -> Option<Rc<RefCell<dyn Shape>>> {
        if let ShapeType::Lookup(l) = shape.shape_type() {
            return l.resolve(self.qs.clone())
        }
        return None
    }

    fn quad_store(&self) -> Option<Rc<RefCell<dyn QuadStore>>> {
        return Some(self.qs.clone())
    }
}

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
        println!("Lookup build_iterator()"); 
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

    fn shape_type(&mut self) -> ShapeType {
        ShapeType::Lookup(self)
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

    fn shape_type(&mut self) -> ShapeType {
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

    fn shape_type(&mut self) -> ShapeType {
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

    fn shape_type(&mut self) -> ShapeType {
        ShapeType::AllNodes
    }
}


///////////////////////////////////////////////

#[derive(Clone)]
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
            if let ShapeType::Null = c.borrow_mut().shape_type() {
                return None
            }
            let v = c.borrow_mut().optimize(ctx, r);
            if v.is_none() {
                continue;
            }
            if let ShapeType::Null = v.as_ref().unwrap().borrow_mut().shape_type() {
                return None
            }
            self.0[i] = v.unwrap();
        }

        return None
    }

    fn shape_type(&mut self) -> ShapeType {
        ShapeType::Intersect(self)
    }
}


///////////////////////////////////////////////


pub struct NodesFrom {
    dir: Direction,
    quads: Rc<RefCell<dyn Shape>>
}

impl Shape for NodesFrom {
    fn build_iterator(&self, qs: Rc<RefCell<dyn QuadStore>>) -> Rc<RefCell<dyn iterator::Shape>> {
        if let ShapeType::Null = self.quads.borrow_mut().shape_type() {
            return iterator::Null::new() 
        }
        let sub = self.quads.borrow().build_iterator(qs.clone());
        if let Direction::Any = self.dir {
            panic!("direction is not set");
        }
        return HasA::new(qs.clone(), sub, self.dir.clone())
    }

    fn optimize(&mut self, ctx: &Context, r: Option<&dyn Optimizer>) -> Option<Rc<RefCell<dyn Shape>>> {
        // TODO: Implement
        return None
    }

    fn shape_type(&mut self) -> ShapeType {
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
        println!("Quad Filter build_iterator()");

        if self.values.is_none() {
            println!("Quad Filter self.values.is_none()");
            return iterator::Null::new() 
        }

        if let Some(v) = one(self.values.clone().unwrap()) {
            println!("Quad Filter Some(v) = one(self.values.clone().unwrap())");
            return qs.borrow().quad_iterator(&self.dir, &v)
        }

        if let Direction::Any = self.dir {
            panic!("direction is not set")
        }

        let sub = self.values.clone().unwrap().borrow().build_iterator(qs.clone());

        println!("Quad Filter LinksTo::new(qs.clone(), sub, self.dir.clone())");

        LinksTo::new(qs.clone(), sub, self.dir.clone())
    }

    fn optimize(&mut self, ctx: &Context, r: Option<&dyn Optimizer>) -> Option<Rc<RefCell<dyn Shape>>> {
        // TODO: Implement
        return None
    }

    fn shape_type(&mut self) -> ShapeType {
        ShapeType::QuadFilter
    }
}


///////////////////////////////////////////////


pub struct Quads(pub Vec<QuadFilter>);

impl Quads {
    fn interset(&mut self, mut q: Vec<QuadFilter>) {
        self.0.append(&mut q)
    }
}

impl Shape for Quads {
    fn build_iterator(&self, qs: Rc<RefCell<dyn QuadStore>>) -> Rc<RefCell<dyn iterator::Shape>> {
        println!("Quads build_iterator() {:?} self.0.len()", self.0.len());

        if self.0.is_empty() {
            return iterator::Null::new() 
        }

        let mut its:Vec<Rc<RefCell<dyn iterator::Shape>>> = Vec::new();

        for f in &self.0 {
            its.push(f.build_iterator(qs.clone()));
        }

        if its.len() == 1 {
            return its[0].clone()
        }

        return iterator::and::And::new(its)
    }

    fn optimize(&mut self, ctx: &Context, r: Option<&dyn Optimizer>) -> Option<Rc<RefCell<dyn Shape>>> {
        // TODO: Implement
        return None
    }

    fn shape_type(&mut self) -> ShapeType {
        ShapeType::Quads
    }
}


///////////////////////////////////////////////

pub struct Save {
    tags: Vec<String>,
    from: Option<Rc<RefCell<dyn Shape>>>
}

impl Shape for Save {
    fn build_iterator(&self, qs: Rc<RefCell<dyn QuadStore>>) -> Rc<RefCell<dyn iterator::Shape>> {
        if let None = self.from {
            return iterator::Null::new() 
        }

        let it = self.from.as_ref().unwrap().borrow().build_iterator(qs.clone());
        if !self.tags.is_empty() {
            return iterator::save::Save::new(it, self.tags.clone())
        }
        return it
    }

    fn optimize(&mut self, ctx: &Context, r: Option<&dyn Optimizer>) -> Option<Rc<RefCell<dyn Shape>>> {
        // TODO: Implement
        return None
    }

    fn shape_type(&mut self) -> ShapeType {
        ShapeType::Save
    }
}


///////////////////////////////////////////////

pub struct Union(pub Vec<Rc<RefCell<dyn Shape>>>);

impl Shape for Union {
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

        return iterator::or::Or::new(sub)
    }

    fn optimize(&mut self, ctx: &Context, r: Option<&dyn Optimizer>) -> Option<Rc<RefCell<dyn Shape>>> {
        // TODO: Implement
        return None
    }

    fn shape_type(&mut self) -> ShapeType {
        ShapeType::Union
    }
}



///////////////////////////////////////////////


pub struct Recursive {
    path: path::Path,
    r#in: Rc<RefCell<dyn Shape>>,
    max_depth: i32, 
    tags: Vec<String>
}

impl Shape for Recursive {
    fn build_iterator(&self, qs: Rc<RefCell<dyn QuadStore>>) -> Rc<RefCell<dyn iterator::Shape>> {
        let r#in = self.r#in.borrow().build_iterator(qs.clone());
        let it = iterator::recursive::Recursive::new(r#in, path::MorphismForPath::new(self.path.clone(), qs.clone()), self.max_depth);
        for s in &self.tags {
            it.borrow_mut().add_depth_tag(s.clone());
        }
        return it
    }

    fn optimize(&mut self, ctx: &Context, r: Option<&dyn Optimizer>) -> Option<Rc<RefCell<dyn Shape>>> {
        return None
    }

    fn shape_type(&mut self) -> ShapeType {
        ShapeType::Recursive
    }
}

///////////////////////////////////////////////
#[derive(Clone)]
pub struct IteratorShape {
    pub it: Rc<RefCell<dyn iterator::Shape>>,
    pub sent: bool
}

impl Shape for IteratorShape {
    fn build_iterator(&self, qs: Rc<RefCell<dyn QuadStore>>) -> Rc<RefCell<dyn iterator::Shape>> {
        // TODO: Implement
        return iterator::Null::new()
    }

    fn optimize(&mut self, ctx: &Context, r: Option<&dyn Optimizer>) -> Option<Rc<RefCell<dyn Shape>>> {
        // TODO: Implement
        return None
    }

    fn shape_type(&mut self) -> ShapeType {
        ShapeType::IteratorShape
    }
}

///////////////////////////////////////////////
#[derive(Clone)]
pub struct IntersectOpt {
    pub sub: Intersect,
    pub opt: Vec<Rc<RefCell<dyn Shape>>>
}

impl IntersectOpt {
    pub fn add(&mut self, arr: Vec<Rc<RefCell<dyn Shape>>>) {

    }

    pub fn add_optional(&mut self, arr: Vec<Rc<RefCell<dyn Shape>>>) {

    }
}

impl Shape for IntersectOpt {
    fn build_iterator(&self, qs: Rc<RefCell<dyn QuadStore>>) -> Rc<RefCell<dyn iterator::Shape>> {
        // TODO: Implement
        return iterator::Null::new()
    }

    fn optimize(&mut self, ctx: &Context, r: Option<&dyn Optimizer>) -> Option<Rc<RefCell<dyn Shape>>> {
        // TODO: Implement
        return None
    }

    fn shape_type(&mut self) -> ShapeType {
        ShapeType::IntersectOpt(self)
    }
}


///////////////////////////////////////////////


pub trait ValueFilter {
    fn build_iterator(&self, qs: Rc<RefCell<dyn QuadStore>>, shape: Rc<RefCell<dyn iterator::Shape>>) -> Rc<RefCell<dyn iterator::Shape>>;
}

pub struct Regexp {
    re: Regex
}

impl Regexp {
    pub fn new(pattern: String) -> Regexp {
        let re = Regex::new(&pattern).unwrap();
        Regexp {
            re
        }
    }
}

impl ValueFilter for Regexp {
    fn build_iterator(&self, qs: Rc<RefCell<dyn QuadStore>>, it: Rc<RefCell<dyn iterator::Shape>>) -> Rc<RefCell<dyn iterator::Shape>> {
        iterator::value_filter::RegexValueFilter::new(it, qs, self.re.clone())
    }
}


pub struct Wildcard {
    pattern: String
}

fn quote_meta(s: &String) -> String {
    let special = "\\.+*?()|[]{}^$";
    let v:Vec<String> = s.chars().map(|x| {
        if special.contains(x) {
            return format!("\\{}", x)
        } else {
            return x.to_string()
        }
    }).collect();
    return v.join("")
}

impl Wildcard {
    pub fn new(pattern: String) -> Wildcard {
        Wildcard {
            pattern
        }
    }

    fn regexp(&self) -> String {
        let any = '%';

        let mut pattern = quote_meta(&self.pattern);
        
        if !pattern.starts_with(any) {
            pattern = format!("^{}", pattern);
        } else {
            pattern = pattern.trim_start_matches(any).to_string();
        }
        
        if !pattern.ends_with(any) {
            pattern = format!("{}$", pattern);
        } else {
            pattern = pattern.trim_end_matches(any).to_string();
        }

        pattern = pattern.replace(any, ".*");
        pattern = pattern.replace("\\?", ".");

        return pattern
    }
}

impl ValueFilter for Wildcard {
    fn build_iterator(&self, qs: Rc<RefCell<dyn QuadStore>>, it: Rc<RefCell<dyn iterator::Shape>>) -> Rc<RefCell<dyn iterator::Shape>> {
        if self.pattern.is_empty() {
            return iterator::Null::new()
        } else if self.pattern.trim_matches('%').is_empty() {
            return it
        }

        let re = Regex::new(&self.regexp()).unwrap();

        iterator::value_filter::RegexValueFilter::new(it, qs, re)
    }
}


pub struct Comparison {
    op: iterator::value_filter::Operator,
    val: Value
}

impl Comparison {
    pub fn new(op: iterator::value_filter::Operator, val: Value) -> Comparison {
        Comparison {
            op,
            val
        }
    }
}

impl ValueFilter for Comparison {
    fn build_iterator(&self, qs: Rc<RefCell<dyn QuadStore>>, it: Rc<RefCell<dyn iterator::Shape>>) -> Rc<RefCell<dyn iterator::Shape>> {
        iterator::value_filter::ComparisonValueFilter::new(it, self.op.clone(), self.val.clone(), qs)
    }
}



pub struct Filter {
    from: Rc<RefCell<dyn Shape>>,
    filters: Vec<Rc<dyn ValueFilter>>
}

impl Filter {
    pub fn new(nodes: Rc<RefCell<dyn Shape>>, filters: Vec<Rc<dyn ValueFilter>>) -> Rc<RefCell<dyn Shape>> {
        if filters.is_empty() {
            return nodes
        }
        if let ShapeType::Filter(s) = nodes.borrow_mut().shape_type() {
            let mut f = s.filters.clone();
            f.extend(filters);

            return Rc::new(RefCell::new(Filter {
                from: s.from.clone(),
                filters: f
            }))
        }
        return Rc::new(RefCell::new(Filter {
            from: nodes,
            filters
        }))
    }
}

impl Shape for Filter {
    fn build_iterator(&self, qs: Rc<RefCell<dyn QuadStore>>) -> Rc<RefCell<dyn iterator::Shape>> {
        let mut it = self.from.borrow().build_iterator(qs.clone());
        for f in &self.filters {
            it = f.build_iterator(qs.clone(), it)
        }
        return it
    }

    fn optimize(&mut self, ctx: &Context, r: Option<&dyn Optimizer>) -> Option<Rc<RefCell<dyn Shape>>> {
        return None
    }

    fn shape_type(&mut self) -> ShapeType {
        ShapeType::Filter(self)
    }
}

///////////////////////////////////////////////



pub fn interset_optional(main: Rc<RefCell<dyn Shape>>, opt: Rc<RefCell<dyn Shape>>) -> Rc<RefCell<dyn Shape>>  {
    let mut optional:Vec<Rc<RefCell<dyn Shape>>> = match opt.borrow_mut().shape_type() {
        ShapeType::IntersectOpt(io) => {
            let mut v = Vec::new();
            io.sub.0.iter().for_each(|x| v.push(x.clone()));
            io.opt.iter().for_each(|x| v.push(x.clone()));
            v
        },
        _ => vec![opt.clone()]
    };

    if optional.is_empty() {
        return main.clone()
    }
    
    match main.borrow_mut().shape_type() {
        ShapeType::Intersect(i) => {
            return Rc::new(RefCell::new(IntersectOpt{
                sub: i.clone(),
                opt: optional
            }))
        },
        ShapeType::IntersectOpt(io) => {
            io.opt.append(&mut optional);
            return main.clone()
        },
        _ => {
            return Rc::new(RefCell::new(IntersectOpt{
                sub: Intersect(vec![main.clone()]),
                opt: optional
            }))
        }
    }
}


fn optimize(ctx: & Context, qs: Rc<RefCell<dyn QuadStore>>, shape:Rc<RefCell<dyn Shape>>) -> Option<Rc<RefCell<dyn Shape>>> {
    shape.borrow_mut().optimize(ctx, Some(&ResolveValues{qs}))
}

pub fn build_iterator(ctx: & Context, qs: Rc<RefCell<dyn QuadStore>>, shape:Rc<RefCell<dyn Shape>>) -> Rc<RefCell<dyn iterator::Shape>>{
    let s = optimize(ctx, qs.clone(), shape.clone());
    let s = match s { Some(new_s) => new_s, None => shape.clone() };
    let a = s.borrow();
    a.build_iterator(qs.clone())
}

// buildOut() from query/shape/path.go
pub fn new_in_out(from:Rc<RefCell<dyn Shape>>, mut via:Rc<RefCell<dyn Shape>>, labels:Option<Rc<RefCell<dyn Shape>>>, tags:Vec<String>, r#in: bool) -> Rc<RefCell<dyn Shape>> {
   println!("new_in_out");
   
    let start = if r#in { Direction::Object } else { Direction::Subject };
    let goal = if r#in { Direction::Subject } else { Direction::Object };

    if !tags.is_empty() {
        via = Rc::new(RefCell::new(Save {
            tags: tags,
            from: Some(via)
        }));
    }

    let quads = Rc::new(RefCell::new(Quads(Vec::new())));

    // if from.shape_type != AllNodes
    match from.borrow_mut().shape_type() {
        ShapeType::AllNodes => {},
        _ => {
            quads.borrow_mut().0.push(QuadFilter {
                dir: start,
                values: Some(from.clone())
            });
        }
    };

    // if via.shape_type != AllNodes
    match via.borrow_mut().shape_type() {
        ShapeType::AllNodes => {},
        _ => {
            quads.borrow_mut().0.push(QuadFilter {
                dir: Direction::Predicate,
                values: Some(via.clone())
            });
        }
    };

    if labels.is_some() {
        if let ShapeType::AllNodes = labels.as_ref().unwrap().borrow_mut().shape_type() {
            quads.borrow_mut().0.push(QuadFilter {
                dir: Direction::Label,
                values: Some(labels.clone().unwrap())
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
    if let ShapeType::Fixed(f) = shape.borrow_mut().shape_type() {
        if f.0.len() == 1 {
            return Some(f.0[0].clone())
        }
    }
    return None
}

