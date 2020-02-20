use super::{Shape, ShapeType, Base, Index, Scanner, Costs};
use super::super::refs;
use super::super::value::{Value};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use io_context::Context;
use std::fmt;

pub struct ValueFilter {
    sub: Rc<RefCell<dyn Shape>>,
    filter: fn(Value) -> Result<bool, String>,
    qs: Rc<dyn refs::Namer>
}

impl ValueFilter {
    pub fn new(qs: Rc<dyn refs::Namer>, sub: Rc<RefCell<dyn Shape>>, filter: fn(Value) -> Result<bool, String>) -> Rc<RefCell<ValueFilter>> {
        Rc::new(RefCell::new( ValueFilter {
            sub,
            filter, 
            qs
        }))
    }
}

impl fmt::Display for ValueFilter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ValueFilter")
    }
}


impl Shape for ValueFilter {

    fn iterate(&self) -> Rc<RefCell<dyn Scanner>> {
        ValueFilterNext::new(self.qs.clone(), self.sub.borrow().iterate(), self.filter)
    }

    fn lookup(&self) -> Rc<RefCell<dyn Index>> {
        ValueFilterContains::new(self.qs.clone(), self.sub.borrow().lookup(), self.filter)
    }

    fn stats(&mut self, ctx: &Context) -> Result<Costs, String> {
        let mut st = self.sub.borrow_mut().stats(ctx)?;
        st.size.value = st.size.value/2 + 1;
        st.size.exact = false;
        return Ok(st);
    }

    fn optimize(&mut self, ctx: &Context) -> Option<Rc<RefCell<dyn Shape>>> {
        let new_sub = self.sub.borrow_mut().optimize(ctx);
        if new_sub.is_some() {
            self.sub = new_sub.unwrap();
        }
        return None
    }

    fn sub_iterators(&self) -> Option<Vec<Rc<RefCell<dyn Shape>>>> {
        Some(vec![self.sub.clone()])
    }

    fn shape_type(&mut self) -> ShapeType {
        ShapeType::ValueFilter
    }
}



struct ValueFilterNext {
    sub: Rc<RefCell<dyn Scanner>>,
    filter: fn(Value) -> Result<bool, String>,
    qs: Rc<dyn refs::Namer>,
    result: Option<refs::Ref>,
    err: Option<String>
}

impl ValueFilterNext {
    fn new(qs: Rc<dyn refs::Namer>, sub: Rc<RefCell<dyn Scanner>>, filter: fn(Value) -> Result<bool, String>) -> Rc<RefCell<ValueFilterNext>> {
       Rc::new(RefCell::new( ValueFilterNext {
           sub,
           filter,
           qs,
           result: None,
           err: None
       }))
    }

    fn do_filter(&mut self, val: &refs::Ref) -> bool {
        let qval = self.qs.name_of(val);
        if qval.is_none() {
            self.err = Some("no name for val".to_string());
            return false
        }
        let res = (self.filter)(qval.unwrap());
        match res {
            Result::Ok(r) => r,
            Result::Err(e) => {
                self.err = Some(e);
                return false
            }
        }
    }
}

impl fmt::Display for ValueFilterNext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ValueFilterNext")
    }
}

impl Base for ValueFilterNext {
    fn tag_results(&self, tags: &mut HashMap<String, refs::Ref>) {
        self.sub.borrow().tag_results(tags)
    }

    fn result(&self) -> Option<refs::Ref> {
        return self.result.clone()
    }

    fn next_path(&mut self, ctx: &Context) -> bool {
        return self.sub.borrow_mut().next_path(ctx)
    }

    fn err(&self) -> Option<String> {
        return self.err.clone()
    }

    fn close(&mut self) -> Result<(), String> {
        return self.sub.borrow_mut().close()
    }
}

impl Scanner for ValueFilterNext {
    fn next(&mut self, ctx: &Context) -> bool {
        while self.sub.borrow_mut().next(ctx) {
            let val = self.sub.borrow().result().unwrap();
            if self.do_filter(&val) {
                self.result = Some(val);
                return true
            }
        }
        self.err = self.sub.borrow().err();
        return false
    }
}



struct ValueFilterContains {
    sub: Rc<RefCell<dyn Index>>,
    filter: fn(Value) -> Result<bool, String>,
    qs: Rc<dyn refs::Namer>,
    result: Option<refs::Ref>,
    err: Option<String>
}

impl ValueFilterContains {
    fn new(qs: Rc<dyn refs::Namer>, sub: Rc<RefCell<dyn Index>>, filter: fn(Value) -> Result<bool, String>) -> Rc<RefCell<ValueFilterContains>> {
        Rc::new(RefCell::new( ValueFilterContains {
            sub, 
            filter,
            qs,
            result: None,
            err: None
        }))
    }

    fn do_filter(&self, val: &refs::Ref) -> bool {
        let qval = self.qs.name_of(val);
        if qval.is_none() {
            return false
        }
        let res = (self.filter)(qval.unwrap());
        match res {
            Result::Ok(r) => r,
            Result::Err(_) => false
        }
    }
}

impl fmt::Display for ValueFilterContains {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ValueFilterContains")
    }
}

impl Base for ValueFilterContains {
    fn tag_results(&self, tags: &mut HashMap<String, refs::Ref>) {
        self.sub.borrow().tag_results(tags)
    }

    fn result(&self) -> Option<refs::Ref> {
        self.result.clone()
    }

    fn next_path(&mut self, ctx: &Context) -> bool {
        self.sub.borrow_mut().next_path(ctx)
    }

    fn err(&self) -> Option<String> {
        self.err.clone()
    }

    fn close(&mut self) -> Result<(), String> {
        self.sub.borrow_mut().close()
    }
}

impl Index for ValueFilterContains {
    fn contains(&mut self, ctx: &Context, v:&refs::Ref) -> bool {
        if !self.do_filter(v) {
            return false
        }
        let ok = self.sub.borrow_mut().contains(ctx, v);
        if !ok {
            self.err = self.sub.borrow().err();
        }
        return ok
    }
}