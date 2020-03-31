use crate::graph::value::Value;
use crate::graph::refs::{Size, Ref, Content};
use crate::graph::iterator::{Base, Scanner, Index, Shape, Costs, ShapeType};

use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;

use std::collections::BTreeMap;
use super::quadstore::{PrimStore, Primitive};
use std::ops::Bound::{Excluded, Unbounded};

use std::sync::{Arc, RwLock};

pub struct MemStoreAllIterator {
    all: Arc<RwLock<dyn PrimStore>>,
    maxid: i64,
    nodes: bool
}

impl MemStoreAllIterator {
    pub fn new(all: Arc<RwLock<dyn PrimStore>>, maxid: i64, nodes: bool) -> Rc<RefCell<MemStoreAllIterator>> {
  
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

    fn stats(&mut self) -> Result<Costs, String> {
        let all = self.all.read().unwrap();

        Ok(Costs {
            contains_cost: 1,
            next_cost: 1,
            size: Size {
                value: all.len() as i64,
                exact: true
            }
        })
    }

    fn optimize(&mut self) -> Option<Rc<RefCell<dyn Shape>>> {
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
    all: Arc<RwLock<dyn PrimStore>>,
    maxid: i64,
    nodes: bool,
    done: bool,
    cur: Option<i64>
}



impl MemStoreAllIteratorNext {
    pub fn new(all: Arc<RwLock<dyn PrimStore>>, maxid: i64, nodes: bool) -> Rc<RefCell<MemStoreAllIteratorNext>> {
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
                k: Value::from(c),
                content: Content::None
            }),
            None => None
        }
    }

    fn next_path(&mut self) -> bool {
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
    fn next(&mut self) -> bool {
        
        if self.done {
            println!("MemStoreAllIteratorNext called after done");
            return false
        }

        // TODO: This is ridiculous, there has to be a way to just use a single iterator.

        let lam = |(k, v):&(&i64, &Primitive)| {

            let is_node = v.is_node();

            if **k > self.maxid {
                return false
            } else if self.nodes && is_node {
                return true
            } else if !self.nodes && !is_node {
                return true
            } 

            return false
            
        };

        let all = self.all.read().unwrap();

        self.cur = if !self.done && self.cur.is_none() {
            all.iter().filter(lam).map(|(k, _)| *k).next()
        } else {
            all.range((Excluded(self.cur.unwrap()), Unbounded)).filter(lam).map(|(k, _)| *k).next()
        };

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
    all: Arc<RwLock<dyn PrimStore>>,
    maxid: i64,
    nodes: bool,
    cur: Option<i64>,
    done: bool
}

impl MemStoreAllIteratorContains {
    pub fn new(all: Arc<RwLock<dyn PrimStore>>, maxid: i64, nodes: bool) -> Rc<RefCell<MemStoreAllIteratorContains>> {
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
                k: Value::from(c),
                content: Content::None
            }),
            None => None
        }
    }

    fn next_path(&mut self) -> bool {
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
    fn contains(&mut self, v:&Ref) -> bool {
        if self.done {
            return false
        }

        let all = self.all.read().unwrap();

        let id = if let Some(k) = v.key() { k.as_i64() } else { None };

        // TODO: if id > maxid
        match id {
            Some(i) => {
                match all.get(&i) {
                    Some(_) => {
                        self.cur = Some(i);
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
