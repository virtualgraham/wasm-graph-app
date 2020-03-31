use super::{Shape, Base, Index, Scanner, Costs, ShapeType};
use super::super::refs;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;

pub struct Limit {
    limit: i64,
    it: Rc<RefCell<dyn Shape>>
}

impl Limit {
    pub fn new(it: Rc<RefCell<dyn Shape>>, limit: i64) -> Rc<RefCell<Limit>> {
        Rc::new(RefCell::new(Limit {
            limit,
            it: it.clone()
        }))
    }
}


impl fmt::Display for Limit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Limit({})", self.limit)
    }
}


impl Shape for Limit {
    fn iterate(&self) -> Rc<RefCell<dyn Scanner>> {
        return LimitNext::new(&self.it.borrow().iterate(), self.limit)
    }

    fn lookup(&self) -> Rc<RefCell<dyn Index>> {
        return LimitContains::new(&self.it.borrow().lookup(), self.limit)
    }

    fn stats(&mut self) -> Result<Costs, String> {
        let mut st = self.it.borrow_mut().stats()?;
        if self.limit > 0 && st.size.value > self.limit {
            st.size.value = self.limit
        }
        return Ok(st)
    }

    fn optimize(&mut self) -> Option<Rc<RefCell<dyn Shape>>> {
        let nit = self.it.borrow_mut().optimize();
        if self.limit <= 0 {
            return nit
        }
        self.it = nit.unwrap();
        return None
    }

    fn sub_iterators(&self) -> Option<Vec<Rc<RefCell<dyn Shape>>>> {
        return Some(vec![self.it.clone()])
    }

    fn shape_type(&mut self) -> ShapeType {
        ShapeType::Limit
    }
}


struct LimitNext {
    limit: i64,
    count: i64,
    it: Rc<RefCell<dyn Scanner>>
}

impl LimitNext {
    fn new(it: &Rc<RefCell<dyn Scanner>>, limit: i64) -> Rc<RefCell<LimitNext>> {
        Rc::new(RefCell::new(LimitNext {
            limit,
            count: 0,
            it: it.clone()
        }))
    }
}

impl fmt::Display for LimitNext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "LimitNext({})", self.limit)
    }
}

impl Base for LimitNext {

    fn tag_results(&self, tags: &mut HashMap<String, refs::Ref>) {
        self.it.borrow().tag_results(tags)
    }

    fn result(&self) -> Option<refs::Ref> {
        self.it.borrow().result()
    }

    fn next_path(&mut self) -> bool {
        if self.limit > 0 && self.count >= self.limit {
            return false
        }
        if self.it.borrow_mut().next_path() {
            self.count += 1;
            return true
        }
        return false
    }

    fn err(&self) -> Option<String> {
        self.it.borrow().err()
    }

    fn close(&mut self) -> Result<(), String> {
        self.it.borrow_mut().close()
    }
}

impl Scanner for LimitNext {
    fn next(&mut self) -> bool {
        if self.limit > 0 && self.count >= self.limit {
            return false
        }
        if self.it.borrow_mut().next() {
            self.count += 1;
            return true
        }
        return false
    }
}


struct LimitContains {
    limit: i64,
    count: i64,
    it: Rc<RefCell<dyn Index>>
}

impl LimitContains {
    fn new(it: &Rc<RefCell<dyn Index>>, limit: i64) -> Rc<RefCell<LimitContains>> {
        Rc::new(RefCell::new(LimitContains {
            limit,
            count: 0,
            it: it.clone()
        }))
    }
}

impl fmt::Display for LimitContains {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "LimitContains({})", self.limit)
    }
}

impl Base for LimitContains {

    fn tag_results(&self, tags: &mut HashMap<String, refs::Ref>) {
        self.it.borrow().tag_results(tags)
    }

    fn result(&self) -> Option<refs::Ref> {
        self.it.borrow().result()
    }

    fn next_path(&mut self) -> bool {
        if self.limit > 0 && self.count >= self.limit {
            return false
        }
        if self.it.borrow_mut().next_path() {
            self.count += 1;
            return true;
        }
        false
    }

    fn err(&self) -> Option<String> {
        self.it.borrow().err()
    }

    fn close(&mut self) -> Result<(), String> {
        self.it.borrow_mut().close()
    }
}

impl Index for LimitContains {
    fn contains(&mut self, v:&refs::Ref) -> bool {
        if self.limit > 0 && self.count >= self.limit {
            return false
        }
        if self.it.borrow_mut().contains(v) {
            self.count += 1;
            return true;
        }
        false
    }
}