

use crate::graph::value::Value;
use crate::graph::refs::{Size, Ref, Content};
use crate::graph::iterator::{Base, Scanner, Index, Shape, Costs, ShapeType};
use crate::graph::quad::{Direction};

use io_context::Context;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;

use std::collections::BTreeMap;
use super::quadstore::Primitive;
use std::ops::Bound::{Excluded, Unbounded};



pub struct MemStoreAllIterator {
    all: Rc<RefCell<BTreeMap<i64, Primitive>>>,
    maxid: i64,
    nodes: bool
}

impl MemStoreAllIterator {
    pub fn new(all: Rc<RefCell<BTreeMap<i64, Primitive>>>, maxid: i64, nodes: bool) -> Rc<RefCell<MemStoreAllIterator>> {
  
        Rc::new(RefCell::new(MemStoreAllIterator {
            all,
            maxid,
            nodes
        }))
    }
}

impl Shape for MemStoreAllIterator {

    fn iterate(&self) -> Rc<RefCell<dyn Scanner>> {
        MemStoreAllIteratorNext::new(self.all.clone(), self.maxid, self.nodes)
    }

    fn lookup(&self) -> Rc<RefCell<dyn Index>> {
        MemStoreAllIteratorContains::new(self.all.clone(), self.maxid, self.nodes)
    }

    fn stats(&mut self, ctx: &Context) -> Result<Costs, String> {
        Ok(Costs {
            contains_cost: 1,
            next_cost: 1,
            size: Size {
                value: self.all.borrow().len() as i64,
                exact: true
            }
        })
    }

    fn optimize(&mut self, ctx: &Context) -> Option<Rc<RefCell<dyn Shape>>> {
        None
    }

    fn sub_iterators(&self) -> Option<Vec<Rc<RefCell<dyn Shape>>>> {
        None
    }

    fn shape_type(&mut self) -> ShapeType {
        ShapeType::MemStoreIterator
    }

}


impl fmt::Display for MemStoreAllIterator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MemStoreAllIterator")
    }
}




pub struct MemStoreAllIteratorNext {
    all: Rc<RefCell<BTreeMap<i64, Primitive>>>,
    maxid: i64,
    nodes: bool,
    done: bool,
    cur: Option<i64>
}



impl MemStoreAllIteratorNext {
    pub fn new(all: Rc<RefCell<BTreeMap<i64, Primitive>>>, maxid: i64, nodes: bool) -> Rc<RefCell<MemStoreAllIteratorNext>> {
        


        Rc::new(RefCell::new(MemStoreAllIteratorNext {
            all,
            maxid,
            nodes,
            done: false,
            cur: None
        }))
    }
}

impl Base for MemStoreAllIteratorNext {
    fn tag_results(&self, tags: &mut HashMap<String, Ref>) {}

    fn result(&self) -> Option<Ref> {
        match self.cur {
            Some(c) => Some(Ref {
                key: Value::from(c),
                content: Content::None
            }),
            None => None
        }
    }

    fn next_path(&mut self, ctx: &Context) -> bool {
        false
    }

    fn err(&self) -> Option<String> {
        None
    }

    fn close(&mut self) -> Result<(), String> {
        self.done = true;
        Ok(())
    }
}

impl Scanner for MemStoreAllIteratorNext {
    fn next(&mut self, ctx: &Context) -> bool {
        
        // TODO: This is ridiculous, there has to be a way to just use a single iterator.

        let a = &*self.all.borrow();

        let q = !self.done && self.cur.is_none();

        let iter:Box<dyn Iterator<Item = (&i64, &Primitive)>> = if q {
            Box::new(a.iter())
        } else {
            Box::new(a.range((Excluded(self.cur.unwrap()), Unbounded)))
        };

        self.cur = iter.filter(|(k, v)| {

            let is_node = v.is_node();

            if **k > self.maxid {
                return false
            } else if self.nodes && is_node {
                return true
            } else if !self.nodes && !is_node {
                return true
            } 

            return false
            
        }).map(|(k, _)| *k).next();
        
        println!("MemStoreAllIteratorNext {:?}", self.cur);

        if !self.cur.is_some() {
            self.done = true;
            return false
        }

        return true
    }
}

impl fmt::Display for MemStoreAllIteratorNext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MemStoreAllIteratorNext")
    }
}




pub struct MemStoreAllIteratorContains {
    all: Rc<RefCell<BTreeMap<i64, Primitive>>>,
    maxid: i64,
    nodes: bool,
    cur: Option<i64>,
    done: bool
}

impl MemStoreAllIteratorContains {
    pub fn new(all: Rc<RefCell<BTreeMap<i64, Primitive>>>, maxid: i64, nodes: bool) -> Rc<RefCell<MemStoreAllIteratorContains>> {
        Rc::new(RefCell::new(MemStoreAllIteratorContains {
            all,
            maxid,
            nodes,
            cur: None,
            done: false
        }))
    }
}

impl Base for MemStoreAllIteratorContains {
    fn tag_results(&self, tags: &mut HashMap<String, Ref>) {}

    fn result(&self) -> Option<Ref> {
        match self.cur {
            Some(c) => Some(Ref {
                key: Value::from(c),
                content: Content::None
            }),
            None => None
        }
    }

    fn next_path(&mut self, ctx: &Context) -> bool {
        false
    }

    fn err(&self) -> Option<String> {
        None
    }

    fn close(&mut self) -> Result<(), String> {
        self.done = true;
        Ok(())
    }
}

impl Index for MemStoreAllIteratorContains {
    fn contains(&mut self, ctx: &Context, v:&Ref) -> bool {
        if self.done {
            return false
        }

        let id = v.key.as_i64();

        match id {
            Some(i) => {
                match self.all.borrow().get(&i) {
                    Some(p) => {
                        self.cur = Some(p.id);
                        return true
                    },
                    None => {
                        self.cur = None;
                        return false
                    }
                }
            },
            None => false
        }
    }
}

impl fmt::Display for MemStoreAllIteratorContains {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MemStoreAllIteratorContains")
    }
}
