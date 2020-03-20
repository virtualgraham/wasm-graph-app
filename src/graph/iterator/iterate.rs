use std::rc::Rc;
use std::cell::RefCell;
use io_context::Context;
use super::{Shape, Scanner};
use super::refs::Ref;
use super::super::super::query::gizmo::Session;
use super::super::quad::QuadStore;
use std::collections::HashMap;

pub struct Chain {
    ctx: Rc<RefCell<Context>>,
    s: Rc<RefCell<dyn Shape>>,
    it: Option<Rc<RefCell<dyn Scanner>>>,
    qs: Option<Rc<RefCell<dyn QuadStore>>>,
    paths: bool,
    optimize: bool, 
    limit: i64,
    n: i64
}

impl Chain {
    pub fn new(ctx: Rc<RefCell<Context>>, it: Rc<RefCell<dyn Shape>>) -> Chain {
        Chain {
            ctx,
            s: it,
            it: None,
            qs: None,
            paths: true,
            optimize: true,
            limit: -1,
            n: 0
        }
    }

    pub fn next(&mut self) -> bool {
        if let Some(_) = self.ctx.borrow().done() {
            return false
        }

        let ok = (self.limit < 0 || self.n < self.limit) && self.it.as_ref().unwrap().borrow_mut().next(&*self.ctx.borrow());
        if ok {
            self.n += 1;
        }
        return ok
    }

    pub fn next_path(&mut self) -> bool {
        if let Some(_) = self.ctx.borrow().done() {
            return false
        }

        let ok = (self.limit < 0 || self.n < self.limit) && self.it.as_ref().unwrap().borrow_mut().next_path(&*self.ctx.borrow());
        if ok {
            self.n += 1;
        }
        return ok
    }

    pub fn start(&mut self) {
        if self.optimize {
            let ctx = &*self.ctx.borrow();
            let shape = self.s.clone().borrow_mut().optimize(ctx);

            if let Some(s) = shape {
                self.s = s;
            }
        }
        self.it = Some(self.s.borrow().iterate());
    }

    pub fn end(&mut self) {
        let i = &mut*self.it.as_ref().unwrap().borrow_mut();
        i.close();
    }

    pub fn limit(mut self, n:i64) -> Self {
        self.limit = n;
        return self
    }

    pub fn paths(mut self, enable:bool) -> Self {
        self.paths = enable;
        return self
    }

    pub fn tag_each(&mut self, session: &mut Session) -> Result<(), String> {
        self.start();

        let mut mn = 0;

        while self.next() {
            if let Some(reason) = self.ctx.borrow().done() {
                return Err(reason.to_string());
            }
            
            let mut tags = HashMap::new();
            self.it.as_ref().unwrap().borrow().tag_results(&mut tags);
            let n = tags.len();
            if n > mn {
                mn = n;
            }
            session.do_tag(tags);

            while self.next_path() {
                if let Some(reason) = self.ctx.borrow().done() {
                    return Err(reason.to_string());
                }
                
                let mut tags = HashMap::new();
                self.it.as_ref().unwrap().borrow().tag_results(&mut tags);
                let n = tags.len();
                if n > mn {
                    mn = n;
                }
                session.do_tag(tags);
            }
        }

        self.end();

        match self.it.as_ref().unwrap().borrow().err() { Some(e) => Err(e), None => Ok(())}
    }

    
}
