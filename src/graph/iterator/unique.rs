use super::{Shape, ShapeType, Base, Index, Scanner, Costs};
use super::super::refs;
use super::super::value::{Value};
use std::collections::HashSet;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use io_context::Context;
use std::fmt;

pub struct Unique {
    sub_it: Rc<RefCell<dyn Shape>>,
}

impl Unique {
    pub fn new(sub_it: Rc<RefCell<dyn Shape>>,) -> Rc<RefCell<Unique>> {
        Rc::new(RefCell::new( Unique {
            sub_it
        }))
    }
}

impl fmt::Display for Unique {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Unique")
    }
}

const UNIQUENESS_FACTOR:i64 = 2;

impl Shape for Unique {

    fn iterate(&self) -> Rc<RefCell<dyn Scanner>> {
        UniqueNext::new(self.sub_it.borrow().iterate())
    }

    fn lookup(&self) -> Rc<RefCell<dyn Index>> {
        UniqueContains::new(self.sub_it.borrow().lookup())
    }

    fn stats(&mut self, ctx: &Context) -> Result<Costs, String> {
        let sub_stats = self.sub_it.borrow_mut().stats(ctx)?;
        return Ok(Costs {
            next_cost: sub_stats.next_cost * UNIQUENESS_FACTOR,
            contains_cost: sub_stats.contains_cost,
            size: refs::Size {
                value: sub_stats.size.value / UNIQUENESS_FACTOR,
                exact: false
            }
        })
    }

    fn optimize(&mut self, ctx: &Context) -> Option<Rc<RefCell<dyn Shape>>> {
        let new_it = self.sub_it.borrow_mut().optimize(ctx);
        if new_it.is_some() {
            self.sub_it = new_it.unwrap()
        }
        return None
    }

    fn sub_iterators(&self) -> Option<Vec<Rc<RefCell<dyn Shape>>>> {
        Some(vec![self.sub_it.clone()])
    }

    fn shape_type(&mut self) -> ShapeType {
        ShapeType::Unique
    }
}



struct UniqueNext {
    sub_it: Rc<RefCell<dyn Scanner>>,
    result: Option<refs::Ref>,
    err: Option<String>,
    seen: HashSet<Value>
}

impl UniqueNext {
    fn new(sub_it: Rc<RefCell<dyn Scanner>>) -> Rc<RefCell<UniqueNext>> {
       Rc::new(RefCell::new(UniqueNext {
           sub_it,
           result: None,
           err: None,
           seen: HashSet::new()
       }))
    }
}

impl fmt::Display for UniqueNext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "UniqueNext")
    }
}

impl Base for UniqueNext {

    fn tag_results(&self, tags: &mut HashMap<String, refs::Ref>) {
        self.sub_it.borrow().tag_results(tags)
    }

    fn result(&self) -> Option<refs::Ref> {
        self.result.clone()
    }

    #[allow(unused)]
    fn next_path(&mut self, ctx: &Context) -> bool {
        false
    }

    fn err(&self) -> Option<String> {
        self.err.clone()
    }

    fn close(&mut self) -> Result<(), String> {
        self.seen = HashSet::new();
        self.sub_it.borrow_mut().close()
    }
}

impl Scanner for UniqueNext {
    fn next(&mut self, ctx: &Context) -> bool {
        while self.sub_it.borrow_mut().next(ctx) {
            let curr = self.sub_it.borrow().result();
            let key = curr.as_ref().unwrap().key().clone();
            if key.is_some() && !self.seen.contains(key.unwrap()) {
                self.result = curr.clone();
                self.seen.insert(key.unwrap().clone());
                return true
            }
        }
        self.err = self.sub_it.borrow().err();
        return false
    }
}



struct UniqueContains {
    sub_it: Rc<RefCell<dyn Index>>,
}

impl UniqueContains {
    fn new(sub_it: Rc<RefCell<dyn Index>>) -> Rc<RefCell<UniqueContains>> {
       Rc::new(RefCell::new(UniqueContains {
           sub_it
       }))
    }
}

impl fmt::Display for UniqueContains {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "UniqueContains")
    }
}

impl Base for UniqueContains {
    fn tag_results(&self, tags: &mut HashMap<String, refs::Ref>) {
        self.sub_it.borrow().tag_results(tags)
    }

    fn result(&self) -> Option<refs::Ref> {
        self.sub_it.borrow().result()
    }

    #[allow(unused)]
    fn next_path(&mut self, ctx: &Context) -> bool {
        false
    }

    fn err(&self) -> Option<String> {
        self.sub_it.borrow().err()
    }

    fn close(&mut self) -> Result<(), String> {
        self.sub_it.borrow_mut().close()
    }
}

impl Index for UniqueContains {
    fn contains(&mut self, ctx: &Context, v:&refs::Ref) -> bool {
        self.sub_it.borrow_mut().contains(ctx, v)
    }
}