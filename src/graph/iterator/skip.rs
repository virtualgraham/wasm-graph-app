use super::{Shape, Base, Index, Scanner, Costs, ShapeType};
use super::super::refs;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use io_context::Context;
use std::fmt;

pub struct Skip {
    skip: i64,
    primary_it: Rc<RefCell<dyn Shape>>,
}

impl Skip {
    pub fn new(primary_it: Rc<RefCell<dyn Shape>>, skip: i64) -> Rc<RefCell<Skip>> {
        Rc::new(RefCell::new(Skip {
            skip,
            primary_it
        }))
    }
}


impl fmt::Display for Skip {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Skip")
    }
}


impl Shape for Skip {
    fn iterate(&self) -> Rc<RefCell<dyn Scanner>> {
        SkipNext::new(self.primary_it.borrow().iterate(), self.skip)
    }

    fn lookup(&self) -> Rc<RefCell<dyn Index>> {
        SkipContains::new(self.primary_it.borrow().lookup(), self.skip)
    }

    fn stats(&mut self, ctx: &Context) -> Result<Costs, String> {
        let mut primary_stats = self.primary_it.borrow_mut().stats(ctx)?;
        if primary_stats.size.exact {
            primary_stats.size.value -= self.skip;
            if primary_stats.size.value < 0 {
                primary_stats.size.value = 0;
            }
        }
        return Ok(primary_stats)
    }

    fn optimize(&mut self, ctx: &Context) -> Option<Rc<RefCell<dyn Shape>>> {
        let optimized_primary_it = self.primary_it.borrow_mut().optimize(ctx);
        if self.skip == 0 {
            return optimized_primary_it
        }
        self.primary_it = optimized_primary_it.unwrap();
        return None
    }

    fn sub_iterators(&self) -> Option<Vec<Rc<RefCell<dyn Shape>>>> {
        Some(vec![self.primary_it.clone()])
    }

    fn shape_type(&mut self) -> ShapeType {
        ShapeType::Skip
    }
}



struct SkipNext {
    skip: i64,
    skipped: i64,
    primary_it: Rc<RefCell<dyn Scanner>>,
}

impl SkipNext {
    fn new(primary_it: Rc<RefCell<dyn Scanner>>, skip: i64) -> Rc<RefCell<SkipNext>> {
       Rc::new(RefCell::new(SkipNext {
           skip,
           skipped: 0,
           primary_it
       }))
    }
}

impl fmt::Display for SkipNext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SkipNext")
    }
}

impl Base for SkipNext {
    fn tag_results(&self, tags: &mut HashMap<String, refs::Ref>) {
        self.primary_it.borrow().tag_results(tags)
    }

    fn result(&self) -> Option<refs::Ref> {
        self.primary_it.borrow().result()
    }

    fn next_path(&mut self, ctx: &Context) -> bool {
        while self.skipped < self.skip {
            if !self.primary_it.borrow_mut().next_path(ctx) {
                return false
            }
            self.skipped += 1;
        }
        return self.primary_it.borrow_mut().next_path(ctx)
    }

    fn err(&self) -> Option<String> {
        self.primary_it.borrow().err()
    }

    fn close(&mut self) -> Result<(), String> {
        self.primary_it.borrow_mut().close()
    }
}

impl Scanner for SkipNext {
    fn next(&mut self, ctx: &Context) -> bool {
        while self.skipped < self.skip {
            if !self.primary_it.borrow_mut().next(ctx) {
                return false
            }
            self.skipped += 1;
        }
        if self.primary_it.borrow_mut().next(ctx) {
            return true
        }
        return false
    }
}


struct SkipContains {
    skip: i64,
    skipped: i64,
    primary_it: Rc<RefCell<dyn Index>>,
}

impl SkipContains {
    fn new(primary_it: Rc<RefCell<dyn Index>>, skip: i64) -> Rc<RefCell<SkipContains>> {
        Rc::new(RefCell::new(SkipContains {
            skip,
            skipped: 0,
            primary_it
        }))
    }
}

impl fmt::Display for SkipContains {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SkipContains")
    }
}

impl Base for SkipContains {

    fn tag_results(&self, dst: &mut HashMap<String, refs::Ref>) {
        self.primary_it.borrow_mut().tag_results(dst)
    }

    fn result(&self) -> Option<refs::Ref> {
        self.primary_it.borrow().result()
    }

    fn next_path(&mut self, ctx: &Context) -> bool {
        while self.skipped < self.skip {
            if !self.primary_it.borrow_mut().next_path(ctx) {
                return false
            }
            self.skipped += 1;
        }
        return self.primary_it.borrow_mut().next_path(ctx)
    }

    fn err(&self) -> Option<String> {
        self.primary_it.borrow().err()
    }

    fn close(&mut self) -> Result<(), String> {
        self.primary_it.borrow_mut().close()
    }
}

impl Index for SkipContains {
    fn contains(&mut self, ctx: &Context, val:&refs::Ref) -> bool {
        let mut in_next_path = false;

        while self.skipped <= self.skip {

            in_next_path = false;
            if !self.primary_it.borrow_mut().contains(ctx, val) {
                return false
            }
            self.skipped += 1;

            if self.skipped <= self.skip {
                in_next_path = true;
                if !self.primary_it.borrow_mut().next_path(ctx) {
                    return false
                }
                self.skipped += 1;
            
                while self.skipped <= self.skip {
                    if !self.primary_it.borrow_mut().next_path(ctx) {
                        return false;
                    }
                    self.skipped += 1;
                }
            }
        }

        if in_next_path && self.primary_it.borrow_mut().next_path(ctx) {
            return true
        }

        return self.primary_it.borrow_mut().contains(ctx, val)
    }
}