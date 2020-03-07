use super::{Shape, Base, Index, Scanner, Costs, Null, ShapeType};
use super::super::refs;
use super::super::value::Value;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use io_context::Context;
use std::fmt;

pub struct Fixed {
    pub values: Rc<RefCell<Vec<refs::Ref>>>}

impl Fixed {
    pub fn new(vals: Vec<refs::Ref>) -> Rc<RefCell<Fixed>> {
        Rc::new(RefCell::new(Fixed{
            values: Rc::new(RefCell::new(vals))
        }))
    }

    pub fn add(&mut self, v: refs::Ref) {
        self.values.borrow_mut().push(v);
    }

    fn values(&mut self) -> Rc<RefCell<Vec<refs::Ref>>> {
        self.values.clone()
    }
}


impl fmt::Display for Fixed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Fixed({})", self.values.borrow().len())
    }
}


impl Shape for Fixed {
    fn iterate(&self) -> Rc<RefCell<dyn Scanner>> {
        FixedNext::new(self.values.clone())
    }

    fn lookup(&self) -> Rc<RefCell<dyn Index>> {
        FixedContains::new(self.values.clone())
    }

    #[allow(unused)]
    fn stats(&mut self, ctx: &Context) -> Result<Costs, String> {
        Ok(Costs {
            contains_cost: 1,
            next_cost: 1,
            size: refs::Size {
                value: self.values.borrow().len() as i64,
                exact: true
            }
        })
    }

    #[allow(unused)]
    fn optimize(&mut self, ctx: &Context) -> Option<Rc<RefCell<dyn Shape>>> {
        if self.values.borrow().len() == 1 && self.values.borrow().get(0).is_none() {
            return Some(Null::new());
        }
        return None
    }

    fn sub_iterators(&self) -> Option<Vec<Rc<RefCell<dyn Shape>>>> {
        None
    }

    fn shape_type(&mut self) -> ShapeType {
        ShapeType::Fixed(self)
    }
}



struct FixedNext {
    values: Rc<RefCell<Vec<refs::Ref>>>,
    ind: usize,
    result: Option<refs::Ref>
}

impl FixedNext {
    fn new(vals: Rc<RefCell<Vec<refs::Ref>>>) -> Rc<RefCell<FixedNext>> {
        Rc::new(RefCell::new(FixedNext {
            values: vals,
            ind: 0,
            result: None
        }))
    }
}

impl fmt::Display for FixedNext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "FixedNext")
    }
}

impl Base for FixedNext {

    #[allow(unused)]
    fn tag_results(&self, tags: &mut HashMap<String, refs::Ref>) {}

    fn result(&self) -> Option<refs::Ref> {
        self.result.clone()
    }

    #[allow(unused)]
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

impl Scanner for FixedNext {
    #[allow(unused)]
    fn next(&mut self, ctx: &Context) -> bool {
        if self.ind >= self.values.borrow().len() {
            return false
        }
        self.result = Some(self.values.borrow()[self.ind].clone());
        self.ind += 1;
        true
    }
}



struct FixedContains {
    values: Rc<RefCell<Vec<refs::Ref>>>,
    keys: Vec<Value>,
    result: Option<refs::Ref>
}

impl FixedContains {
   fn new(values: Rc<RefCell<Vec<refs::Ref>>>) -> Rc<RefCell<FixedContains>> {
        Rc::new(RefCell::new(FixedContains {
            keys: values.borrow().iter().map(|r| r.key.clone()).collect(),
            values: values.clone(),
            result: None
        }))
   }   
}

impl fmt::Display for FixedContains {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "FixedContains")
    }
}

impl Base for FixedContains {

    #[allow(unused)]
    fn tag_results(&self, tags: &mut HashMap<String, refs::Ref>) {}

    fn result(&self) -> Option<refs::Ref> {
        self.result.clone()
    }

    #[allow(unused)]
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

impl Index for FixedContains {
    #[allow(unused)]
    fn contains(&mut self, ctx: &Context, v:&refs::Ref) -> bool {
        for (i, x) in self.keys.iter().enumerate() {
            if *x == v.key {
                self.result = Some(self.values.borrow()[i].clone());
                return true
            }
        }
        false
    }
}