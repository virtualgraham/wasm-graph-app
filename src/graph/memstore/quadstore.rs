use crate::graph::value::Value;
use crate::graph::refs::{Size, Ref, Namer, Content};
use crate::graph::iterator::{Shape, Null};
use crate::graph::quad::{QuadStore, Quad, Direction, Stats, Delta, IgnoreOptions, Procedure};

use io_context::Context;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

use super::iterator::MemStoreIterator;
use super::all_iterator::MemStoreAllIterator;


#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct QuadDirectionKey {
    direction: i8,
    value_id: i64,
    quad_id: i64
}

impl QuadDirectionKey {
    pub fn new(value_id: i64, direction: &Direction, quad_id: i64) -> QuadDirectionKey {
        QuadDirectionKey {
            direction: direction.to_byte(),
            value_id,
            quad_id
        }
    }
}

struct QuadDirectionIndex {
    index: BTreeSet<QuadDirectionKey>,
}

impl QuadDirectionIndex {

    fn new() -> QuadDirectionIndex {
        QuadDirectionIndex {
            index: BTreeSet::new()
        }
    }

    // get all quad_ids that have the given value_id at the given location
    fn get(&self, d: &Direction, value_id: i64) -> BTreeSet<i64> {
        let lower_bound = QuadDirectionKey::new(value_id, d, 0);
        self.index.range(lower_bound..).take_while(|k| {
            k.value_id == value_id
        }).map(|k| k.quad_id).collect()
    }

    fn insert(&mut self, value_id: i64, d: &Direction, quad_id: i64) {
        self.index.insert(QuadDirectionKey::new(value_id, d, quad_id));
    }

    fn remove(&mut self, value_id: i64, d: &Direction, quad_id: i64) {
        self.index.remove(&QuadDirectionKey::new(value_id, d, quad_id));
    }
}


pub enum PrimitiveContent {
    Value(Value),
    Quad(InternalQuad)
}

pub struct Primitive {
    pub id: i64,
    pub refs: i32,
    pub content: PrimitiveContent
}

impl Primitive {
    pub fn new_value(v: Value) -> Primitive {
        Primitive {
            id: 0,
            content: PrimitiveContent::Value(v),
            refs: 0
        }
    }

    pub fn new_quad(q: InternalQuad) -> Primitive {
        Primitive {
            id: 0,
            content: PrimitiveContent::Quad(q),
            refs: 0
        }
    }

    pub fn unwrap_value(&self) -> &Value {
        if let PrimitiveContent::Value(v) = &self.content {
            return &v
        } else {
            panic!("Primitive does not contain value")
        }
    }

    pub fn unwrap_quad(&self) -> &InternalQuad {
        if let PrimitiveContent::Quad(q) = &self.content {
            return &q
        } else {
            panic!("Primitive does not contain quad")
        }
    }

    pub fn is_node(&self) -> bool {
        if let PrimitiveContent::Value(_) = self.content {
            return true
        }

        return false
    }
}

#[derive(PartialEq, Hash, Clone, Debug)]
pub struct InternalQuad {
    s: i64,
    p: i64,
    o: i64,
    l: i64,
}

impl Eq for InternalQuad {}

impl InternalQuad {
    fn dir(&self, dir: &Direction) -> i64 {
        match dir {
            Direction::Subject => self.s,
            Direction::Predicate => self.p,
            Direction::Object => self.o,
            Direction::Label => self.l
        }
    }

    fn set_dir(&mut self, dir: &Direction, vid: i64) {
        match dir {
            Direction::Subject => self.s = vid,
            Direction::Predicate => self.p = vid,
            Direction::Object => self.o = vid,
            Direction::Label => self.l = vid,
        };
    }
}


pub struct MemStore {
    vals: HashMap<Value, i64>, // value to value_id
    quads: HashMap<InternalQuad, i64>, // quad to quad_id
    prim: Rc<RefCell<BTreeMap<i64, Primitive>>>, // value_id or quad_id to value or quad
    index: QuadDirectionIndex, // value_id and direction to quad id
    last: i64, // keeps track of ids for values and quads
    horizon: i64 // keeps track of ids for transactions
}

impl MemStore {

    pub fn new() -> MemStore {
        MemStore {
            vals: HashMap::new(),
            quads: HashMap::new(),
            prim: Rc::new(RefCell::new(BTreeMap::new())),
            index: QuadDirectionIndex::new(),
            last: 0,
            horizon: 0
        }
    }

    fn add_primitive(&mut self, mut p: Primitive) -> i64 {
        self.last += 1;
        let id = self.last;
        p.id = id;
        p.refs = 1;
        self.prim.borrow_mut().insert(id, p);
        return id
    }

    fn resolve_val(&mut self, v: &Value, add: bool) -> Option<i64> {
        if let Value::Undefined = v {
            return None
        }
        
        let id = self.vals.get(v);
        
        if id.is_some() || !add {
            // if the value exsists and we are adding it, increment refs
            if id.is_some() && add {
                self.prim.borrow_mut().get_mut(id.unwrap()).as_mut().unwrap().refs += 1;
            }
            // return val_id
            return id.map(|x| *x)
        }

        // value is new and we are adding it
        let id = self.add_primitive(Primitive::new_value(v.clone()));
        self.vals.insert(v.clone(), id);

        return Some(id)
    }

    
    fn resolve_quad(&mut self, q: &Quad, add: bool) -> Option<InternalQuad> {
        let mut p = InternalQuad{s: 0, p: 0, o: 0, l: 0};

        // find all value ids for each direction of quad
        for dir in Direction::iterator() {
            let v = q.get(dir);
            if let Value::Undefined = v {
                continue
            }
            let vid = self.resolve_val(v, add);
            if  let Some(i) = vid {
                p.set_dir(dir, i);
            } else {
                // if any value is not found or undefined return zero value internal quad
                return None
            }
        }

        return Some(p)
    }

    fn resolve_quad_default(&mut self, q: &Quad, add: bool) -> InternalQuad {
        match self.resolve_quad(q, add) {
            Some(q) => q,
            None => InternalQuad{s: 0, p: 0, o: 0, l: 0}
        }
    }

    fn find_quad(&mut self, q: &Quad) -> Option<i64> {
        let quad = self.resolve_quad(q, false);
        if let Some(q) = quad {
            if let Some(id) = self.quads.get(&q) {
                return Some(*id)
            }
        }
        None
    }

    fn delete_quad_nodes(&mut self, q: &InternalQuad) {
        for dir in Direction::iterator() {
            let id = q.dir(dir);
            if id == 0 {
                continue
            }

            let mut delete = false;

            if let Some(p) = self.prim.borrow_mut().get_mut(&id) {
                p.refs -= 1;
                if p.refs < 0 {
                    panic!("remove of delete node");
                } else if p.refs == 0 {
                    delete = true;
                }
            }

            if delete {
                self.delete(id);
            }
        }
    }


    fn delete(&mut self, id: i64) -> bool {

        let mut quad:Option<InternalQuad> = None;
 
        if let Some(p) = self.prim.borrow().get(&id) {
            if p.is_node() {
                self.vals.remove(p.unwrap_value());
            } else {
                quad = Some(p.unwrap_quad().clone());
            }
        } else {
            return false
        }
        
        self.prim.borrow_mut().remove(&id);
        
        if let Some(q) = quad {
            for d in Direction::iterator() {
                self.index.remove(q.dir(d), d, id.clone());
            }

            self.quads.remove(&q);

            self.delete_quad_nodes(&q);
        }

        return true
    }


    fn add_quad(&mut self, q: Quad) -> i64 {
        // get value_ids for each direction
        let p = self.resolve_quad_default(&q, false);

        // get quad id
        let id = self.quads.get(&p);

        // if id already exsists, the quad therefor exsists already. return the id
        if let Some(i) = id {
            return *i
        }

        // get value_ids for each direction, this time inserting the values as neccecery
        let p = self.resolve_quad_default(&q, true);

        // add value primitive
        let pr = Primitive::new_quad(p.clone());
        let id = self.add_primitive(pr);
        
        // add quad
        self.quads.insert(p.clone(), id);

        // add to index
        for d in Direction::iterator() {
            self.index.insert(p.dir(d), d, id);
        }

        return id;
    }


    fn lookup_val(&self, id: &i64) -> Option<Value> {

        match self.prim.borrow().get(id) {
            Some(p) => {
                match &p.content {
                    PrimitiveContent::Value(v) => Some(v.clone()),
                    _ => None
                }
            },
            None => None
        }
    }


    fn internal_quad(&self, r: &Ref) -> Option<InternalQuad> {
        match self.prim.borrow().get(r.key.as_i64().as_ref().unwrap()) {
            Some(p) => {
                match &p.content {
                    PrimitiveContent::Quad(q) => Some(q.clone()),
                    _ => None
                }
            },
            None => None
        }
    }

    fn lookup_quad_dirs(&self, p: InternalQuad) -> Quad {
        let mut q = Quad::new_undefined_vals();
        for dir in Direction::iterator() {
            let vid = p.dir(dir);
            if vid == 0 {
                continue
            }
            let val = self.lookup_val(&vid);
            if let Some(v) = val {
                q.set_val(dir, v);
            }
        }
        return q
    }
}

impl Namer for MemStore {
    fn value_of(&self, v: &Value) -> Option<Ref> {
        if let Value::Undefined = v {
            return None
        }
        let id = self.vals.get(v);
        match id {
            Some(i) => Some(Ref {
                key: Value::from(*i),
                content: Content::None
            }),
            None => None
        } 
    }

    fn name_of(&self, key: &Ref) -> Option<Value> {
        if let Content::Value(v) = &key.content {
            return Some(v.clone())
        }

        let n = key.key.as_i64();

        if let Some(i) = n {
            return self.lookup_val(&i)
        } else {
            return None
        }
    }
}

impl QuadStore for MemStore {

    fn quad(&self, r: &Ref) -> Option<Quad> {
        let quad = self.internal_quad(r);
        match quad {
            Some(q) => Some(self.lookup_quad_dirs(q)),
            None => None
        }
    }

    fn quad_iterator(&self, d: &Direction, r: &Ref) -> Rc<RefCell<dyn Shape>> {
        let id = r.key.as_i64();
        
        if let Some(i) = id {

            let quad_ids = self.index.get(d, i);

            if !quad_ids.is_empty() {
                return MemStoreIterator::new(Rc::new(quad_ids), d.clone())
            }
        } 
            
        Null::new()
    }

    fn quad_iterator_size(&self, ctx: &Context, d: &Direction, r: &Ref) -> Result<Size, String> {
        let id = r.key.as_i64();

        if let Some(i) = id {
            let quad_ids = self.index.get(d, i);
            return Ok(Size{value: quad_ids.len() as i64, exact: true})
        }

        return Ok(Size{value: 0, exact: true})
    }
    
    fn quad_direction(&self, r: &Ref, d: &Direction) -> Option<Ref> {
        let quad = self.internal_quad(r);
        match quad {
            Some(q) => {
                let id = q.dir(d);
                if id == 0 {
                    return None
                }
                return Some(Ref {
                    key: Value::from(id),
                    content: Content::None
                })
            }
            None => None
        }
    }
    
    fn stats(&self, ctx: &Context, exact: bool) -> Result<Stats, String> {
        Ok(Stats {
            nodes: Size {
                value: self.vals.len() as i64,
                exact: true
            },
            quads: Size {
                value: self.quads.len() as i64,
                exact: true
            }
        })
    }
    
    fn apply_deltas(&mut self, deltas: Vec<Delta>, ignore_opts: &IgnoreOptions) -> Result<(), String> {
        
        if !ignore_opts.ignore_dup || !ignore_opts.ignore_missing {
            for d in &deltas {
                match d.action {
                    Procedure::Add => {
                        if !ignore_opts.ignore_dup {
                            if let Some(_) = self.find_quad(&d.quad) {
                                return Err("ErrQuadExists".into())
                            }
                        }
                    },
                    Procedure::Delete => {
                        if !ignore_opts.ignore_missing {
                            if let Some(_) = self.find_quad(&d.quad) {
                            } else {
                                return Err("ErrQuadNotExist".into())
                            }
                        }
                    },
                }
            }
        }

        for d in &deltas {
            match &d.action {
                Procedure::Add => {
                   self.add_quad(d.quad.clone());
                },
                Procedure::Delete => {
                   if let Some(id) = self.find_quad(&d.quad) {
                       self.delete(id);
                   }
                }
            }
        }

        self.horizon += 1;

        Ok(())
    }
    
    fn nodes_all_iterator(&self) -> Rc<RefCell<dyn Shape>> {
        MemStoreAllIterator::new(self.prim.clone(), self.last, true)
    }
    
    fn quads_all_iterator(&self) -> Rc<RefCell<dyn Shape>> {
        MemStoreAllIterator::new(self.prim.clone(), self.last, false)
    }
    
    fn close(&self) -> Option<String> {
        None
    }
}