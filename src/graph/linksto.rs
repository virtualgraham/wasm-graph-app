
use super::refs::{Ref, Size};
use super::quad::{Direction, QuadStore};
use super::iterator::{Shape, Scanner, Costs, Index, Base, ShapeType, Null, is_null};
use std::rc::Rc;
use std::cell::RefCell;
use io_context::Context;
use std::collections::HashMap;
use std::fmt;

pub struct LinksTo {
    qs: Rc<RefCell<dyn QuadStore>>,
    primary: Rc<RefCell<dyn Shape>>,
    dir: Direction,
    size: Size
}

impl LinksTo {
    pub fn new(qs: Rc<RefCell<dyn QuadStore>>, primary: Rc<RefCell<dyn Shape>>, dir: Direction) -> Rc<RefCell<LinksTo>> {
        println!("LinksTo dir {:?}", dir);
        Rc::new(RefCell::new(LinksTo {
            qs,
            primary,
            dir,
            size: Size {
                value: 0,
                exact: true
            }
        }))
    }

    fn direction(&self) -> Direction {
        return self.dir.clone()
    }

    fn get_size(&mut self, ctx: &Context) -> Size {
        if self.size.value != 0 {
            return self.size.clone()
        }

        if let ShapeType::Fixed(fixed) = self.primary.borrow_mut().shape_type() {
            let mut size = Size {
                value: 0,
                exact: true
            };

            for v in fixed.values.borrow().iter() {
                let sit = self.qs.borrow().quad_iterator(&self.dir, v);
                let st = sit.borrow_mut().stats(ctx);
                size.value += st.as_ref().unwrap().size.value;
                size.exact = size.exact && st.as_ref().unwrap().size.exact;
            }
            self.size.value = size.value;
            self.size.exact = size.exact;
            return size;
        }

        let stats = self.qs.borrow().stats(ctx, false).unwrap();
        let max_size = stats.quads.value/2 + 1;

        let fanout_factor = 20;
        let st = self.primary.borrow_mut().stats(ctx);
        let mut value = st.unwrap().size.value * fanout_factor;
        if value > max_size {
            value = max_size;
        }
        self.size.value = value;
        self.size.exact = false;

        return self.size.clone();
    }
}


impl fmt::Display for LinksTo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "LinksTo")
    }
}


impl Shape for LinksTo {
    fn iterate(&self) -> Rc<RefCell<dyn Scanner>> {
        LinksToNext::new(self.qs.clone(), self.primary.borrow().iterate(), self.dir.clone())
    }

    fn lookup(&self) -> Rc<RefCell<dyn Index>> {
        LinksToContains::new(self.qs.clone(), self.primary.borrow().lookup(), self.dir.clone())
    }

    fn stats(&mut self, ctx: &Context) -> Result<Costs, String> {
        let subit_stats = self.primary.borrow_mut().stats(ctx).unwrap();
        let check_constant = 1i64;
        let next_contant = 2i64;
        return Ok(Costs {
            next_cost: next_contant + subit_stats.next_cost,
            contains_cost: check_constant + subit_stats.contains_cost,
            size: self.get_size(ctx)
        })
    }

    fn optimize(&mut self, ctx: &Context) -> Option<Rc<RefCell<dyn Shape>>> {
        let new_primary = self.primary.borrow_mut().optimize(ctx);
        if new_primary.is_some() {
            self.primary = new_primary.unwrap();
            if is_null(&self.primary) {
                return Some(self.primary.clone())
            }
        }
        return None
    }

    fn sub_iterators(&self) -> Option<Vec<Rc<RefCell<dyn Shape>>>> {
        Some(vec![self.primary.clone()])
    }

    fn shape_type(&mut self) -> ShapeType {
        ShapeType::LinksTo
    }
}



struct LinksToNext {
    qs: Rc<RefCell<dyn QuadStore>>,
    primary: Rc<RefCell<dyn Scanner>>,
    dir: Direction,
    next_it: Rc<RefCell<dyn Scanner>>,
    result: Option<Ref>,
    err: Option<String>
}


impl LinksToNext {
    fn new(qs: Rc<RefCell<dyn QuadStore>>, primary: Rc<RefCell<dyn Scanner>>, dir: Direction,) -> Rc<RefCell<LinksToNext>> {
        Rc::new(RefCell::new(LinksToNext {
            qs,
            primary,
            dir,
            next_it: Null::new().borrow().iterate(),
            result: None,
            err: None
        }))
    }

    fn direction(&self) -> Direction {
        return self.dir.clone()
    }
}

impl fmt::Display for LinksToNext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "LinksToNext")
    }
}

impl Base for LinksToNext {

    fn tag_results(&self, tags: &mut HashMap<String, Ref>) {
        self.primary.borrow().tag_results(tags)
    }

    fn result(&self) -> Option<Ref> {
        self.result.clone()
    }

    fn next_path(&mut self, ctx: &Context) -> bool {
        let ok = self.primary.borrow_mut().next_path(ctx);
        if !ok {
            self.err = self.primary.borrow().err();
        }
        return ok
    }

    fn err(&self) -> Option<String> {
        self.err.clone()
    }

    fn close(&mut self) -> Result<(), String> {
        let mut res = self.next_it.borrow_mut().close();
        let res2 = self.primary.borrow_mut().close();
        if res2.is_err() && res.is_ok() {
            res = res2;
        }

        return res;
    }
}

impl Scanner for LinksToNext {
    fn next(&mut self, ctx: &Context) -> bool {
        loop {
            if self.next_it.borrow_mut().next(ctx) {
                self.result = self.next_it.borrow().result();
                return true
            }

            self.err = self.next_it.borrow().err();
            if self.err.is_some() {
                return false
            }

            if !self.primary.borrow_mut().next(ctx) {
                self.err = self.primary.borrow().err();
                return false
            }

            let _ = self.next_it.borrow_mut().close();
            self.next_it = self.qs.borrow().quad_iterator(&self.dir, self.primary.borrow().result().as_ref().unwrap()).borrow().iterate();
        }
    }

}



struct LinksToContains {
    qs: Rc<RefCell<dyn QuadStore>>,
    primary: Rc<RefCell<dyn Index>>,
    dir: Direction,
    result: Option<Ref>,
}

impl LinksToContains {
    fn new(qs: Rc<RefCell<dyn QuadStore>>, primary: Rc<RefCell<dyn Index>>, dir: Direction) -> Rc<RefCell<LinksToContains>> {
        Rc::new(RefCell::new(LinksToContains {
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

impl fmt::Display for LinksToContains {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "LinksToContains")
    }
}

impl Base for LinksToContains {

    fn tag_results(&self, tags: &mut HashMap<String, Ref>) {
        self.primary.borrow().tag_results(tags)
    }

    fn result(&self) -> Option<Ref> {
        self.result.clone()
    }

    fn next_path(&mut self, ctx: &Context) -> bool {
        self.primary.borrow_mut().next_path(ctx)
    }

    fn err(&self) -> Option<String> {
        self.primary.borrow().err()
    }

    fn close(&mut self) -> Result<(), String> {
        self.primary.borrow_mut().close()
    }
}

impl Index for LinksToContains {
    fn contains(&mut self, ctx: &Context, val:&Ref) -> bool {
        let node = self.qs.borrow().quad_direction(val, &self.dir);
        match node {
            Some(n) => {
                if self.primary.borrow_mut().contains(ctx, &n) {
                    self.result = Some(val.clone());
                    return true
                }
                return false
            },
            None => {
                return false
            }
        }
    }
}