use super::{Shape, Base, Index, Scanner, Costs, ShapeType};
use super::and::optimize_sub_iterators;
use super::super::refs;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;

pub struct Or {
    is_short_circuiting: bool,
    sub: Vec<Rc<RefCell<dyn Shape>>>,
    cur_ind: Option<usize>,
    result: Option<refs::Ref>,
    err: Option<String>
}

impl Or {
    pub fn new(sub: Vec<Rc<RefCell<dyn Shape>>>) -> Rc<RefCell<Or>> {
        Rc::new(RefCell::new(Or {
            is_short_circuiting: false,
            sub,
            cur_ind: None,
            result: None,
            err: None
        }))
    }

    pub fn new_short_circuit(sub: Vec<Rc<RefCell<dyn Shape>>>) -> Rc<RefCell<Or>> {
        Rc::new(RefCell::new(Or {
            is_short_circuiting: true,
            sub,
            cur_ind: None,
            result: None,
            err: None
        }))
    }

    pub fn add_sub_iterator(&mut self, sub: Rc<RefCell<dyn Shape>>) {
        self.sub.push(sub)
    }
}


impl fmt::Display for Or {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Or")
    }
}


impl Shape for Or {
    fn iterate(&self) -> Rc<RefCell<dyn Scanner>> {
        let mut sub = Vec::new();
        for s in &self.sub {
            sub.push(s.borrow().iterate());
        }
        return OrNext::new(sub, self.is_short_circuiting)
    }

    fn lookup(&self) -> Rc<RefCell<dyn Index>> {
        let mut sub = Vec::new();
        for s in &self.sub {
            sub.push(s.borrow().lookup());
        }
        return OrContains::new(sub, self.is_short_circuiting)
    }

    fn stats(&mut self) -> Result<Costs, String> {
        let mut contains_cost = 0i64;
        let mut next_cost = 0i64;
        let mut size = refs::Size {
            value: 0,
            exact: true
        };

        for sub in &self.sub {
            let stats = sub.borrow_mut().stats()?;
            next_cost += stats.next_cost;
            contains_cost += stats.contains_cost;
            if self.is_short_circuiting {
                if size.value < stats.size.value {
                    size = stats.size;
                }
            } else {
                size.value += stats.size.value;
                size.exact = size.exact && stats.size.exact;
            }
        }

        return Ok(Costs {
            contains_cost,
            next_cost,
            size
        })
    }

    fn optimize(&mut self) -> Option<Rc<RefCell<dyn Shape>>> {
        let old = self.sub_iterators();
        let opt_its = optimize_sub_iterators(&old.unwrap());
        let new_or = Or::new(vec![]);
        new_or.borrow_mut().is_short_circuiting = self.is_short_circuiting;

        for o in opt_its {
            new_or.borrow_mut().add_sub_iterator(o)
        }

        return Some(new_or);
    }

    fn sub_iterators(&self) -> Option<Vec<Rc<RefCell<dyn Shape>>>> {
        Some(self.sub.iter().map(|s| s.clone()).collect())
    }

    fn shape_type(&mut self) -> ShapeType {
        ShapeType::Or
    }
}



struct OrNext {
    short_circuit: bool,
    sub: Vec<Rc<RefCell<dyn Scanner>>>,
    cur_ind: Option<usize>,
    result: Option<refs::Ref>,
    err: Option<String>
}

impl OrNext {
    fn new(sub: Vec<Rc<RefCell<dyn Scanner>>>, short_circuit: bool) -> Rc<RefCell<OrNext>> {
       Rc::new(RefCell::new(OrNext {
           sub: sub,
           cur_ind: None,
           short_circuit,
           result: None,
           err: None
       }))
    }
}

impl fmt::Display for OrNext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "OrNext")
    }
}

impl Base for OrNext {

    fn tag_results(&self, tags: &mut HashMap<String, refs::Ref>) {
        self.sub.get(self.cur_ind.unwrap()).unwrap().borrow().tag_results(tags)
    }

    fn result(&self) -> Option<refs::Ref> {
        return self.result.clone()
    }

    fn next_path(&mut self) -> bool {
        if self.cur_ind.is_some() {
            let curr_it = self.sub.get(self.cur_ind.unwrap()).unwrap();
            let ok = curr_it.borrow_mut().next_path();
            if !ok {
                self.err = curr_it.borrow().err();
            }
            return ok;
        }
        return false
    }

    fn err(&self) -> Option<String> {
        return self.err.clone()
    }

    fn close(&mut self) -> Result<(), String> {
        let mut res: Result<(), String> = Ok(());
        for sub in &self.sub {
            let _res = sub.borrow_mut().close();
            if _res.is_err() && res.is_ok() {
                res = _res;
            }
        }
        return res
    }
}

impl Scanner for OrNext {
    fn next(&mut self) -> bool {
        if self.cur_ind.is_some() && self.cur_ind.unwrap() >= self.sub.len() {
            return false
        }
        let mut first: bool = false;
        loop {
            if self.cur_ind.is_none() {
                self.cur_ind = Some(0);
                first= true;
            }
            let cur_it = self.sub.get(self.cur_ind.unwrap()).unwrap();

            if cur_it.borrow_mut().next() {
                self.result = cur_it.borrow().result();
                return true;
            }

            self.err = cur_it.borrow().err();
            if self.err.is_some() {
                return false;
            }

            if self.short_circuit && !first {
                break
            }
            self.cur_ind = Some(self.cur_ind.unwrap() + 1);
            if self.cur_ind.unwrap() >= self.sub.len() {
                break
            }
        }

        return false
    }
}



struct OrContains {
    short_circuit: bool,
    sub: Vec<Rc<RefCell<dyn Index>>>,
    cur_ind: Option<usize>,
    result: Option<refs::Ref>,
    err: Option<String>
}

impl OrContains {
    fn new(sub: Vec<Rc<RefCell<dyn Index>>>, short_circuit: bool) -> Rc<RefCell<OrContains>> {
        Rc::new(RefCell::new(OrContains {
           sub: sub,
           cur_ind: None,
           short_circuit,
           result: None,
           err: None
       }))
    }

    fn sub_its_contain(&mut self, val:&refs::Ref) -> Result<bool, String> {
        let mut sub_is_good = false;
        for (i, sub) in self.sub.iter().enumerate() {
            sub_is_good = sub.borrow_mut().contains(val);
            if sub_is_good {
                self.cur_ind = Some(i);
                break
            }
            let err = sub.borrow().err();
            if err.is_some() {
                return Err(err.unwrap())
            }
        }
        return Ok(sub_is_good)
    }
}

impl fmt::Display for OrContains {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "OrContains")
    }
}

impl Base for OrContains {

    fn tag_results(&self, tags: &mut HashMap<String, refs::Ref>) {
        self.sub.get(self.cur_ind.unwrap()).unwrap().borrow().tag_results(tags)
    }

    fn result(&self) -> Option<refs::Ref> {
        return self.result.clone()
    }

    fn next_path(&mut self) -> bool {
        if self.cur_ind.is_some() {
            let curr_it = self.sub.get(self.cur_ind.unwrap()).unwrap();
            let ok = curr_it.borrow_mut().next_path();
            if !ok {
                self.err = curr_it.borrow().err();
            }
            return ok
        }
        return false
    }

    fn err(&self) -> Option<String> {
        return self.err.clone()
    }

    fn close(&mut self) -> Result<(), String> {
        let mut res: Result<(), String> = Ok(());
        for sub in &self.sub {
            let _res = sub.borrow_mut().close();
            if _res.is_err() && res.is_ok() {
                res = _res;
            }
        }
        return res;
    }
}

impl Index for OrContains {
    fn contains(&mut self, val:&refs::Ref) -> bool {
       let any_good = self.sub_its_contain(val);
       if let Err(err) = any_good {
            self.err = Some(err);
            return false
       }
       if !any_good.unwrap() {
           return false
       }
       self.result = Some(val.clone());
       return true
    }
}