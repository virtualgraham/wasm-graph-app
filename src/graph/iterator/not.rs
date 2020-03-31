use super::{Shape, Base, Index, Scanner, Costs, ShapeType};
use super::materialize::Materialize;
use super::super::refs;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;

pub struct Not {
    primary: Rc<RefCell<dyn Shape>>,
    all_it: Rc<RefCell<dyn Shape>>
}

impl Not {
    pub fn new(primary: Rc<RefCell<dyn Shape>>, all_it: Rc<RefCell<dyn Shape>>) -> Rc<RefCell<Not>> {
        Rc::new(RefCell::new(Not {
            primary: primary,
            all_it: all_it,
        }))
    }
}


impl fmt::Display for Not {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Not")
    }
}


impl Shape for Not {
    fn iterate(&self) -> Rc<RefCell<dyn Scanner>> {
        NotNext::new(self.primary.borrow().lookup(), self.all_it.borrow().iterate())
    }

    fn lookup(&self) -> Rc<RefCell<dyn Index>> {
        NotContains::new(self.primary.borrow().lookup())
    }

    fn stats(&mut self) -> Result<Costs, String> {
        let primary_stats = self.primary.borrow_mut().stats()?;
        let all_stats = self.all_it.borrow_mut().stats()?;
        return Ok(Costs {
            next_cost: all_stats.next_cost + primary_stats.contains_cost,
            contains_cost: primary_stats.contains_cost,
            size: refs::Size {
                value: all_stats.size.value - primary_stats.size.value,
                exact: false
            }
        })
    }

    fn optimize(&mut self) -> Option<Rc<RefCell<dyn Shape>>> {
        let optimized_primary_it = self.primary.borrow_mut().optimize();
        if optimized_primary_it.is_some() {
            self.primary = optimized_primary_it.unwrap();
        }
        self.primary = Materialize::new(self.primary.clone());
        return None
    }

    fn sub_iterators(&self) -> Option<Vec<Rc<RefCell<dyn Shape>>>> {
        Some(vec![self.primary.clone(), self.all_it.clone()])
    }

    fn shape_type(&mut self) -> ShapeType {
        ShapeType::Not
    }
}



struct NotNext {
    primary_it: Rc<RefCell<dyn Index>>,
    all_it: Rc<RefCell<dyn Scanner>>,
    result: Option<refs::Ref>
}

impl NotNext {
    fn new(primary_it: Rc<RefCell<dyn Index>>, all_it: Rc<RefCell<dyn Scanner>>) -> Rc<RefCell<NotNext>> {
        Rc::new(RefCell::new(NotNext {
            primary_it: primary_it.clone(),
            all_it: all_it.clone(),
            result: None
        }))
    }
}

impl fmt::Display for NotNext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "NotNext")
    }
}

impl Base for NotNext {

    fn tag_results(&self, tags: &mut HashMap<String, refs::Ref>) {
        self.primary_it.borrow().tag_results(tags);
    }

    fn result(&self) -> Option<refs::Ref> {
        return self.result.clone()
    }

    #[allow(unused)]
    fn next_path(&mut self) -> bool {
        false
    }

    fn err(&self) -> Option<String> {
        let err = self.all_it.borrow().err();
        if err.is_some() {
            return err
        }
        let err = self.primary_it.borrow().err();
        if err.is_some() {
            return err
        }
        return None
    }

    fn close(&mut self) -> Result<(), String> {
        let err = self.primary_it.borrow_mut().close();
        let err2 = self.all_it.borrow_mut().close();
        if err2.is_err() && err.is_ok() {
            return err2
        } else if err.is_err() {
            return err
        }
        return Ok(())
    }
}

impl Scanner for NotNext {
    fn next(&mut self) -> bool {
        while self.all_it.borrow_mut().next() {
            let curr = self.all_it.borrow().result();
            if !self.primary_it.borrow_mut().contains(curr.as_ref().unwrap()) {
                self.result = curr;
                return true
            }
        }
        return false
    }

}



struct NotContains {
    primary_it: Rc<RefCell<dyn Index>>,
    result: Option<refs::Ref>,
    err: Option<String>
}

impl NotContains {
    fn new(primary_it: Rc<RefCell<dyn Index>>) -> Rc<RefCell<NotContains>> {
        Rc::new(RefCell::new(NotContains {
            primary_it: primary_it.clone(),
            result: None,
            err: None
        }))
    }
}

impl fmt::Display for NotContains {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "NotContains")
    }
}

impl Base for NotContains {

    fn tag_results(&self, tags: &mut HashMap<String, refs::Ref>) {
        self.primary_it.borrow().tag_results(tags);
    }

    fn result(&self) -> Option<refs::Ref> {
        return self.result.clone()
    }

    #[allow(unused)]
    fn next_path(&mut self) -> bool {
        false
    }

    fn err(&self) -> Option<String> {
        return self.err.clone()
    }

    fn close(&mut self) -> Result<(), String> {
        self.primary_it.borrow_mut().close()
    }
}

impl Index for NotContains {
    fn contains(&mut self, v:&refs::Ref) -> bool {
        if self.primary_it.borrow_mut().contains(v) {
            return false
        }
        self.err = self.primary_it.borrow().err();
        if self.err.is_some() {
            return false
        }
        self.result = Some(v.clone());
        true
    }
}