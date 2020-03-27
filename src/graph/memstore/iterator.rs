use crate::graph::value::Value;
use crate::graph::refs::{Size, Ref, Namer, Content};
use crate::graph::iterator::{Base, Scanner, Index, Shape, Null, Costs, ShapeType};
use crate::graph::quad::{QuadStore, Quad, Direction};

use io_context::Context;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fmt;

use super::quadstore::{MemStore, Primitive};

pub struct MemStoreIterator<'a> {
    tree: &'a BTreeSet<i64>,
    d: Direction
}

impl<'a> MemStoreIterator<'a> {
    pub fn new(tree: &'a BTreeSet<i64>, d: Direction) -> Rc<RefCell<MemStoreIterator>> {
        Rc::new(RefCell::new(MemStoreIterator {
            tree,
            d
        }))
    }
}

impl Shape for MemStoreIterator<'_> {

    fn iterate(&self) -> Rc<RefCell<dyn Scanner>> {
        MemStoreIteratorNext::new(self.tree, self.d)
    }

    fn lookup(&self) -> Rc<RefCell<dyn Index>> {
        MemStoreIteratorContains::new(self.tree, self.d)
    }

    fn stats(&mut self, ctx: &Context) -> Result<Costs, String> {
        Ok(Costs {
            contains_cost: ((self.tree.len() as f64).ln() as i64) + 1,
            next_cost: 1,
            size: Size {
                value: self.tree.len() as i64,
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


impl fmt::Display for MemStoreIterator<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MemStoreIterator")
    }
}


pub struct MemStoreIteratorNext<'a> {
    tree: &'a BTreeSet<i64>,
    iter: Option<Box<dyn Iterator<Item = i64>>>,
    cur: Option<i64>,
    d: Direction
}

impl MemStoreIteratorNext<'_> {
    pub fn new(tree: &BTreeSet<i64>, d: Direction) -> Rc<RefCell<MemStoreIteratorNext>> {
        Rc::new(RefCell::new(MemStoreIteratorNext {
            tree,
            iter: None,
            cur: None,
            d
        }))
    }
}

impl Base for MemStoreIteratorNext<'_> {
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

impl Scanner for MemStoreIteratorNext<'_> {
    fn next(&mut self, ctx: &Context) -> bool {
        if self.iter.is_none() {
            self.iter = Some(Box::new(self.tree.iter()));
        }

        loop {
            let p = self.iter.unwrap().next();
            match p {
                Some(p_) => {
                    self.cur = Some(p_.clone());
                    return true
                }
                None => return false
            }
        }
    }
}

impl fmt::Display for MemStoreIteratorNext<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MemStoreIteratorNext {:?}", self.d)
    }
}




pub struct MemStoreIteratorContains<'a> {
    tree: &'a BTreeSet<i64>,
    cur: Option<i64>,
    d: Direction
}

impl MemStoreIteratorContains<'_> {
    pub fn new(tree: &BTreeSet<i64>, d: Direction) -> Rc<RefCell<MemStoreIteratorContains>> {
        Rc::new(RefCell::new(MemStoreIteratorContains {
            tree,
            cur: None,
            d
        }))
    }
}

impl Base for MemStoreIteratorContains<'_> {
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

impl Index for MemStoreIteratorContains<'_> {
    fn contains(&mut self, ctx: &Context, v:&Ref) -> bool {
        let id = v.unwrap_value().as_i64();
        match id {
            Some(i) => {
                let c = self.tree.contains(&i);
                if c {
                    self.cur = Some(i);
                    return true
                } else {
                    return false
                }
            },
            None => false
        }
    }
}

impl fmt::Display for MemStoreIteratorContains<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MemStoreIteratorContains {:?}", self.d)
    }
}
