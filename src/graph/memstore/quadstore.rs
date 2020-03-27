use crate::graph::value::Value;
use crate::graph::refs::{Size, Ref, Namer, Content};
use crate::graph::iterator::{Shape, Null, Scanner, Index, Costs, ShapeType};
use crate::graph::quad::{QuadStore, Quad, Direction};

use io_context::Context;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fmt;

use super::iterator::MemStoreIterator;

struct QuadDirectionIndex (
    [BTreeMap<i64, BTreeSet<i64>>; 4]
);

fn dir_idx(d: &Direction) -> usize {
    match d {
        Direction::Subject => 0,
        Direction::Predicate => 1,
        Direction::Object => 2,
        Direction::Label => 30
    }
} 

impl QuadDirectionIndex {
    fn get(&self,d: &Direction, id: i64) -> Option<&BTreeSet<i64>> {
        self.0[dir_idx(d)].get(&id)
    }

    fn tree(&self, d: &Direction, id: i64) -> &mut BTreeSet<i64> {
        let tree = self.0[dir_idx(d)].get(&id);
        match tree.as_mut() {
            None => {
                let t = BTreeSet::new();
                let t_ = &mut t;
                self.0[dir_idx(d)].insert(id, t);
                return t_;
            },
            Some(t) => {
                return t
            }
        }
    }
}

pub struct Primitive {
    pub id: i64,
    pub quad: Option<InternalQuad>,
    pub value: Option<Value>,
    pub refs: i32
}

impl Primitive {
    pub fn new_value(v: Value) -> Primitive {
        Primitive {
            id: 0,
            quad: None,
            value: Some(v),
            refs: 0
        }
    }

    pub fn new_quad(q: InternalQuad) -> Primitive {
        Primitive {
            id: 0,
            quad: Some(q),
            value: None,
            refs: 0
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
    last: i64,
    vals: HashMap<Value, i64>,
    quads: HashMap<InternalQuad, i64>,
    prim: BTreeMap<i64, Primitive>,
    index: QuadDirectionIndex,
    horizon: i64
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
            if id.is_some() && add {
                self.prim.get(id.unwrap()).as_mut().unwrap().refs += 1;
            }
            return *id.unwrap()
        }
        let id = self.add_primitive(Primitive::new_value(v.clone()));
        self.vals.insert(v.clone(), id);
        return id
    }

    fn resolve_quad(&self, q: Quad, add: bool) -> InternalQuad {
        let p = InternalQuad{s: 0, p: 0, o: 0, l: 0};

        for dir in Direction::iterator() {
            let v = q.get(dir);
            if let Value::Undefined = v {
                continue
            }
            let vid = self.resolve_val(v, add);
            if  vid != 0 {
                p.set_dir(dir, vid);
            } else {
                return InternalQuad{s: 0, p: 0, o: 0, l: 0}
            }
        }

        return p
    }

    fn indexes_for_quad<'a>(&'a self, q: InternalQuad) -> impl Iterator<Item = &'a mut BTreeSet<i64>> {
        Direction::iterator().filter_map(|dir| {
            let v = q.dir(dir);
            if v == 0 {
                return None
            }
            Some(self.index.tree(dir, v))
        })
    }

    pub fn add_quad(&self, q: Quad) -> i64 {
        let p = self.resolve_quad(q, false);
        let id = self.quads.get(&p);
        if let Some(i) = id {
            return *i
        }
        let p = self.resolve_quad(q, true);
        let pr = Primitive::new_quad(p);
        let id = self.add_primitive(pr);
        self.quads.insert(p, id);

        for t in self.indexes_for_quad(p) {
            t.insert(id);
        }

        return id;
    }

    fn lookup_val(&self, id: &i64) -> Option<Value> {
        let pv = self.prim.get(id);
        match pv {
            Some(p) => p.value,
            None => None
        }
    }

    fn internal_quad(&self, r: &Ref) -> Option<InternalQuad> {
        match self.prim.get(r.key.as_i64().as_ref().unwrap()) {
            Some(p) => p.quad,
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

            let index = self.index.get(d, i);

            if let Some(idx) = index {
                if !idx.is_empty() {
                    return MemStoreIterator::new(idx, d.clone())
                }
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