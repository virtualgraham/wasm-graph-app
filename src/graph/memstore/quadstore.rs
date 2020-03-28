use crate::graph::value::Value;
use crate::graph::refs::{Size, Ref, Namer, Content};
use crate::graph::iterator::{Shape, Null};
use crate::graph::quad::{QuadStore, Quad, Direction};

use io_context::Context;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

use super::iterator::MemStoreIterator;



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
    // get all quad_ids that have the given value_id at the given location
    fn get(&self, d: &Direction, value_id: i64) -> Rc<BTreeSet<i64>> {
        let lower_bound = QuadDirectionKey::new(value_id, d, 0);
        Rc::new(self.index.range(lower_bound..).take_while(|k| {
            k.value_id == value_id
        }).map(|k| k.quad_id).collect())
    }

    fn insert(&mut self, value_id: i64, d: &Direction, quad_id: i64) {
        self.index.insert(QuadDirectionKey::new(value_id, d, quad_id));
    }

    fn remove(&mut self, value_id: i64, d: &Direction, quad_id: i64) {
        self.index.remove(&QuadDirectionKey::new(value_id, d, quad_id));
    }
}

enum PrimitiveContent {
    Value(Value),
    Quad(InternalQuad)
}

pub struct Primitive {
    pub id: i64,
    pub refs: i32,
    pub content: PrimitiveContent,
    
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
        if let PrimitiveContent::Value(v) = self.content {
            return &v
        } else {
            panic!("Primitive does not contain value")
        }
    }

    pub fn unwrap_Quad(&self) -> &InternalQuad {
        if let PrimitiveContent::Quad(q) = self.content {
            return &q
        } else {
            panic!("Primitive does not contain quad")
        }
    }
}

#[derive(PartialEq, Hash, Clone, Debug)]
struct InternalQuad {
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
    prim: BTreeMap<i64, Primitive>, // value_id or quad_id to value or quad
    index: QuadDirectionIndex, // value_id and direction to quad id
    last: i64, // keeps track of ids for values and quads
    horizon: i64 // keeps track of ids for transactions
}

impl MemStore {
    fn add_primitive(&self, p: Primitive) -> i64 {
        self.last += 1;
        let id = self.last;
        p.id = id;
        p.refs = 1;
        self.prim.insert(id, p);
        return id
    }

    fn resolve_val(&self, v: &Value, add: bool) -> i64 {
        if let Value::Undefined = v {
            return 0
        }
        
        let id = self.vals.get(v);
        
        if id.is_some() || !add {
            // if the value exsists and we are adding it, increment refs
            if id.is_some() && add {
                self.prim.get(id.unwrap()).as_mut().unwrap().refs += 1;
            }
            // return val_id
            return *id.unwrap()
        }

        // value is new and we are adding it
        let id = self.add_primitive(Primitive::new_value(v.clone()));
        self.vals.insert(v.clone(), id);

        return id
    }

    
    fn resolve_quad(&self, q: Quad, add: bool) -> InternalQuad {
        let p = InternalQuad{s: 0, p: 0, o: 0, l: 0};

        // find all value ids for each direction of quad
        for dir in Direction::iterator() {
            let v = q.get(dir);
            if let Value::Undefined = v {
                continue
            }
            let vid = self.resolve_val(v, add);
            if  vid != 0 {
                p.set_dir(dir, vid);
            } else {
                // if any value is not found or undefined return zero value internal quad
                return InternalQuad{s: 0, p: 0, o: 0, l: 0}
            }
        }

        return p
    }



    pub fn add_quad(&self, q: Quad) -> i64 {
        // get value_ids for each direction
        let p = self.resolve_quad(q, false);

        // get quad id
        let id = self.quads.get(&p);

        // if id already exsists, the quad therefor exsists already. return the id
        if let Some(i) = id {
            return *i
        }

        // get value_ids for each direction, this time inserting the values as neccecery
        let p = self.resolve_quad(q, true);
        
        // add value primitive
        let pr = Primitive::new_quad(p);
        let id = self.add_primitive(pr);
        
        // add quad
        self.quads.insert(p, id);

        // add to index
        for d in Direction::iterator() {
            self.index.insert(p.dir(d), d, id);
        }

        return id;
    }


    fn lookup_val(&self, id: &i64) -> Option<Value> {
        let pv = self.prim.get(id);
        match pv {
            Some(p) => {
                match p.content {
                    PrimitiveContent::Value(v) => Some(v),
                    _ => None
                }
            },
            None => None
        }
    }

    fn internal_quad(&self, r: &Ref) -> Option<InternalQuad> {
        match self.prim.get(r.key.as_i64().as_ref().unwrap()) {
            Some(p) => {
                match p.content {
                    PrimitiveContent::Quad(q) => Some(q),
                    _ => None
                }
            },
            None => None
        }
    }

    fn lookup_quad_dirs(&self, p: InternalQuad) -> Quad {
        let q = Quad::new_undefined_vals();
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
        if let Content::Value(v) = key.content {
            return Some(v)
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

    fn quad(&self, r: &Ref) -> Quad {
        let quad = self.internal_quad(r);
        match quad {
            Some(q) => self.lookup_quad_dirs(q),
            None => Quad::new_undefined_vals()
        }
    }

    fn quad_iterator(&self, d: &Direction, r: &Ref) -> Rc<RefCell<dyn Shape>> {
        let id = r.key.as_i64();
        
        if let Some(i) = id {

            let quad_ids = self.index.get(d, i);

            if !quad_ids.is_empty() {
                return MemStoreIterator::new(quad_ids, d.clone())
            }
        } 
            
        Null::new()
    }

    fn quad_iterator_size(&self, ctx: &Context, d: &Direction, r: &Ref) -> Result<Size, String> {

    }
    
    fn quad_direction(&self, r: &Ref, d: &Direction) -> Ref {

    }
    
    fn stats(&self, ctx: &Context, exact: bool) -> Result<Stats, String> {

    }
    
    fn apply_deltas(&mut self, deltas: Vec<Delta>, ignore_opts: &IgnoreOptions) -> Result<(), String> {

    }
    
    fn nodes_all_iterator(&self) -> Rc<RefCell<dyn Shape>> {

    }
    
    fn quads_all_iterator(&self) -> Rc<RefCell<dyn Shape>> {

    }
    
    fn close(&self) -> Option<String> {

    }
}