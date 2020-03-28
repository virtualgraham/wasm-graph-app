use crate::graph::value::Value;
use crate::graph::refs::{Size, Ref, Content};
use crate::graph::iterator::{Base, Scanner, Index, Shape, Costs, ShapeType};
use crate::graph::quad::{Direction};

use io_context::Context;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;

use std::collections::BTreeSet;
use std::ops::Bound::{Excluded, Unbounded};

pub struct MemStoreIterator {
    quad_ids: Rc<BTreeSet<i64>>,
    d: Direction
}

impl MemStoreIterator {
    pub fn new(quad_ids: Rc<BTreeSet<i64>>, d: Direction) -> Rc<RefCell<MemStoreIterator>> {
        Rc::new(RefCell::new(MemStoreIterator {
            quad_ids,
            d
        }))
    }
}

impl Shape for MemStoreIterator {

    fn iterate(&self) -> Rc<RefCell<dyn Scanner>> {
        MemStoreIteratorNext::new(self.quad_ids.clone(), self.d.clone())
    }

    fn lookup(&self) -> Rc<RefCell<dyn Index>> {
        MemStoreIteratorContains::new(self.quad_ids.clone(), self.d.clone())
    }

    fn stats(&mut self, ctx: &Context) -> Result<Costs, String> {
        Ok(Costs {
            contains_cost: ((self.quad_ids.len() as f64).ln() as i64) + 1,
            next_cost: 1,
            size: Size {
                value: self.quad_ids.len() as i64,
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


impl fmt::Display for MemStoreIterator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MemStoreIterator {:?}", self.d)
    }
}


pub struct MemStoreIteratorNext {
    quad_ids: Rc<BTreeSet<i64>>,
    d: Direction,
    cur: Option<i64>,
    done: bool
}

impl MemStoreIteratorNext {
    pub fn new(quad_ids: Rc<BTreeSet<i64>>, d: Direction) -> Rc<RefCell<MemStoreIteratorNext>> {
        
        Rc::new(RefCell::new(MemStoreIteratorNext {
            quad_ids,
            d,
            cur: None,
            done: false
        }))
        
    }
}

impl Base for MemStoreIteratorNext {
    fn tag_results(&self, tags: &mut HashMap<String, Ref>) {}

    fn result(&self) -> Option<Ref> {
        match self.cur {
            Some(quad_id) => Some(Ref {
                key: Value::from(quad_id),
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
        Ok(())
    }
}

impl Scanner for MemStoreIteratorNext {
    fn next(&mut self, ctx: &Context) -> bool {

         // TODO: This is ridiculous, there has to be a way to just use a single iterator.

        let q = !self.done && self.cur.is_none();

        let iter:Box<dyn Iterator<Item = &i64>> = if q {
            Box::new(self.quad_ids.iter())
        } else {
            Box::new(
                self.quad_ids.range(
                    (Excluded(self.cur.unwrap()), Unbounded)
                )
            )
        };

        self.cur = iter.map(|quad_id| *quad_id).next();
        
        println!("MemStoreIteratorNext {:?}", self.cur);

        if !self.cur.is_some() {
            self.done = true;
            return false
        }

        return true

    }
}

impl fmt::Display for MemStoreIteratorNext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MemStoreIteratorNext {:?}", self.d)
    }
}




pub struct MemStoreIteratorContains {
    quad_ids: Rc<BTreeSet<i64>>,
    d: Direction,
    cur: Option<i64>
}

impl MemStoreIteratorContains {
    pub fn new(quad_ids: Rc<BTreeSet<i64>>, d: Direction) -> Rc<RefCell<MemStoreIteratorContains>> {
        Rc::new(RefCell::new(MemStoreIteratorContains {
            quad_ids,
            d,
            cur: None
        }))
    }
}

impl Base for MemStoreIteratorContains {
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
        Ok(())
    }
}

impl Index for MemStoreIteratorContains {
    fn contains(&mut self, ctx: &Context, v:&Ref) -> bool {
        let id = v.key.as_i64();
        match id {
            Some(i) => {
                let c = self.quad_ids.contains(&i);
                if c {
                    self.cur = Some(i);
                    return true
                } else {
                    self.cur = None;
                    return false
                }
            },
            None => {
                self.cur = None;
                return false
            }
        }
    }
}

impl fmt::Display for MemStoreIteratorContains {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MemStoreIteratorContains {:?}", self.d)
    }
}
