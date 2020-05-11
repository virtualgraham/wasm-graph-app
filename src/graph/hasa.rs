
use super::refs::{Ref, Size};
use super::quad::{Direction, QuadStore};
use super::iterator::{Shape, Scanner, Costs, Index, Base, ShapeType, is_null, Null};
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;

pub struct HasA {
    qs: Rc<RefCell<dyn QuadStore>>,
    primary: Rc<RefCell<dyn Shape>>,
    dir: Direction,
}

impl HasA {
    pub fn new(qs: Rc<RefCell<dyn QuadStore>>, primary: Rc<RefCell<dyn Shape>>, dir: Direction) -> Rc<RefCell<HasA>> {
        Rc::new(RefCell::new(HasA {
            qs,
            primary,
            dir
        }))
    }

    fn direction(&self) -> Direction {
        return self.dir.clone()
    }
}


impl fmt::Display for HasA {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "HasA")
    }
}


impl Shape for HasA {
    fn iterate(&self) -> Rc<RefCell<dyn Scanner>> {
        let temp = self.primary.borrow().iterate();
        // println!("{}", temp.borrow().to_string());
        HasANext::new(self.qs.clone(), temp, self.dir.clone())
    }

    fn lookup(&self) -> Rc<RefCell<dyn Index>> {
        HasAContains::new(self.qs.clone(), self.primary.borrow().lookup(), self.dir.clone())
    }

    fn stats(&mut self) -> Result<Costs, String> {
        let subit_stats = self.primary.borrow_mut().stats()?;
        let fanin_factor = 1i64;
        let fanout_factor = 30i64;
        let next_constant = 2i64;
        let quad_constant = 1i64;
        return Ok(Costs {
            next_cost: quad_constant + subit_stats.next_cost,
            contains_cost: (fanout_factor * next_constant) * subit_stats.contains_cost,
            size: Size {
                value: fanin_factor * subit_stats.size.value,
                exact: false
            }
        })
    }

    fn optimize(&mut self) -> Option<Rc<RefCell<dyn Shape>>> {
        let new_primary = self.primary.borrow_mut().optimize();
        if new_primary.is_some() {
            self.primary = new_primary.unwrap();
            if is_null(&mut*self.primary.borrow_mut()) {
                return Some(self.primary.clone())
            }
        }
        return None
    }

    fn sub_iterators(&self) -> Option<Vec<Rc<RefCell<dyn Shape>>>> {
        Some(vec![self.primary.clone()])
    }

    fn shape_type(&mut self) -> ShapeType {
        ShapeType::HasA
    }
}



struct HasANext {
    qs: Rc<RefCell<dyn QuadStore>>,
    primary: Rc<RefCell<dyn Scanner>>,
    dir: Direction,
    result: Option<Ref>
}


impl HasANext {
    fn new(qs: Rc<RefCell<dyn QuadStore>>, primary: Rc<RefCell<dyn Scanner>>, dir: Direction) -> Rc<RefCell<HasANext>> {
        Rc::new(RefCell::new(HasANext {
            qs,
            primary,
            dir,
            result: None
        }))
    }

    fn direction(&self) -> Direction {
        return self.dir.clone()
    }
}

impl fmt::Display for HasANext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "HasANext")
    }
}


impl Base for HasANext {

    fn tag_results(&self, tags: &mut HashMap<String, Ref>) {
        self.primary.borrow().tag_results(tags)
    }

    fn result(&self) -> Option<Ref> {
        self.result.clone()
    }

    fn next_path(&mut self) -> bool {
        self.primary.borrow_mut().next_path()
    }

    fn err(&self) -> Option<String> {
        self.primary.borrow().err()
    }

    fn close(&mut self) -> Result<(), String> {
        self.primary.borrow_mut().close()
    }
}

impl Scanner for HasANext {
    fn next(&mut self) -> bool {
        if !self.primary.borrow_mut().next() {
            return false
        }
        
        self.result = match self.qs.borrow().quad_direction(self.primary.borrow().result().as_ref().unwrap(), &self.dir) {
            Some(q) => {
                Some(q)
            },
            None => {
                Some(Ref::none())
            }
        };

        return true
    }
}



struct HasAContains {
    qs: Rc<RefCell<dyn QuadStore>>,
    primary: Rc<RefCell<dyn Index>>,
    dir: Direction,
    results: Option<Rc<RefCell<dyn Scanner>>>,
    result: Option<Ref>,
    err: Option<String>,
}

impl HasAContains {
    fn new(qs: Rc<RefCell<dyn QuadStore>>, primary: Rc<RefCell<dyn Index>>, dir: Direction) -> Rc<RefCell<HasAContains>> {
        Rc::new(RefCell::new(HasAContains {
            qs,
            primary,
            dir,
            results: None,
            result: None,
            err: None
        }))
    }

    fn direction(&self) -> Direction {
        return self.dir.clone()
    }

    fn next_contains(&mut self) -> bool {
        if self.results.is_none() {
            return false
        }
        while self.results.as_ref().unwrap().borrow_mut().next() {
            let link = self.results.as_ref().unwrap().borrow().result();
            // TODO logging
            if self.primary.borrow_mut().contains(link.as_ref().unwrap()) {
                // match self.qs.borrow().quad_direction(link.as_ref().unwrap(), &self.dir) {
                //     Some(q) => {
                //         self.result = Some(q);
                //         return true
                //     },
                //     None => {
                //         panic!("HasANext quad_direction not found");
                //     }
                // }
                if let Some(q) = self.qs.borrow().quad_direction(link.as_ref().unwrap(), &self.dir) {
                    self.result = Some(q);
                    return true
                }
            }
        }
        self.err = self.results.as_ref().unwrap().borrow().err();
        return false;
    }
}

impl fmt::Display for HasAContains {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "HasAContains")
    }
}

impl Base for HasAContains {

    fn tag_results(&self, tags: &mut HashMap<String, Ref>) {
        self.primary.borrow().tag_results(tags)
    }

    fn result(&self) -> Option<Ref> {
        return self.result.clone()
    }

    fn next_path(&mut self) -> bool {
        // TODO: logging
        if self.primary.borrow_mut().next_path() {
            return true
        }
        self.err = self.primary.borrow().err();
        if self.err.is_some() {
            return false
        }

        let result = self.next_contains();
        if self.err.is_some() {
            return false
        }
        // TODO: logging
        return result;
    }

    fn err(&self) -> Option<String> {
        return self.err.clone()
    }

    fn close(&mut self) -> Result<(), String> {
        let mut res = self.primary.borrow_mut().close();
        if self.results.is_some() {
            let res2 = self.results.as_ref().unwrap().borrow_mut().close();
            if res2.is_err() && res.is_ok() {
                res = res2;
            }
        }
        return res
    }
}

impl Index for HasAContains {
    fn contains(&mut self, val:&Ref) -> bool {
        // TODO logging
        if self.results.is_some() {
            let _ = self.results.as_ref().unwrap().borrow_mut().close();
        }
        self.results = Some(self.qs.borrow().quad_iterator(&self.dir, val).borrow().iterate());
        let ok = self.next_contains();
        if self.err.is_some() {
            return false
        }
        return ok
    }
}