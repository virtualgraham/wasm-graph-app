use std::rc::Rc;
use std::cell::RefCell;
use io_context::Context;
use super::{Shape, Scanner};
use super::refs::Ref;
use std::collections::HashMap;


pub struct BaseIterator {
    ctx: Rc<RefCell<Context>>,
    s: Rc<RefCell<dyn Shape>>,
    it: Option<Rc<RefCell<dyn Scanner>>>,
    paths: bool,
    optimize: bool,
    n: i64
}

impl BaseIterator {
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
        i.close().unwrap();
    }


    fn next_val(&mut self) -> bool {
        let ok = self.it.as_ref().unwrap().borrow_mut().next(&*self.ctx.borrow());
        if ok {
            self.n += 1;
        }
        return ok
    }

    fn next_path(&mut self) -> bool {
        let ok = self.it.as_ref().unwrap().borrow_mut().next_path(&*self.ctx.borrow());
        if ok {
            self.n += 1;
        }
        return ok
    }
}


pub struct TagEachIterator {
    base: BaseIterator
}

impl TagEachIterator {
    pub fn new(ctx: Rc<RefCell<Context>>, it: Rc<RefCell<dyn Shape>>, optimize: bool, paths: bool) -> TagEachIterator {
        TagEachIterator {
            base: BaseIterator {
                ctx,
                s: it,
                it: None,
                paths,
                optimize,
                n: 0
            }
        }
    }

    fn do_val(&mut self) -> Option<HashMap<String, Ref>> {

        if self.base.next_val() {
            self.base.paths = true;
            let mut tags = HashMap::new();
            self.base.it.as_ref().unwrap().borrow().tag_results(&mut tags);
            return Some(tags)
        } else {
            self.base.end();
            return None
        }
    }

    fn do_path(&mut self)  -> Option<HashMap<String, Ref>> {

        if self.base.next_path() {
            let mut tags = HashMap::new();
            self.base.it.as_ref().unwrap().borrow().tag_results(&mut tags);
            return Some(tags)
        } else {
            self.base.paths = false;
            return self.do_val()
        }
    }
}



impl Iterator for TagEachIterator {
    type Item = HashMap<String, Ref>;

    fn next(&mut self) -> Option<HashMap<String, Ref>> {

        if !self.base.it.is_some() {
            self.base.start();
        }

        if !self.base.paths {
            return self.do_val()
        } else {
            return self.do_path()
        }
    }
}


pub struct EachIterator {
    base: BaseIterator
}

impl EachIterator {
    pub fn new(ctx: Rc<RefCell<Context>>, it: Rc<RefCell<dyn Shape>>, optimize: bool, paths: bool) -> EachIterator {
        EachIterator {
            base: BaseIterator {
                ctx,
                s: it,
                it: None,
                paths,
                optimize,
                n: 0
            }
        }
    }

    fn do_val(&mut self) -> Option<Ref> {

        if self.base.next_val() {
            self.base.paths = true;
            return self.base.it.as_ref().unwrap().borrow().result()
        } else {
            self.base.end();
            return None
        }
    }

    fn do_path(&mut self)  -> Option<Ref> {

        if self.base.next_path() {
            return self.base.it.as_ref().unwrap().borrow().result()
        } else {
            self.base.paths = false;
            return self.do_val()
        }
    }
}

impl Iterator for EachIterator {
    type Item = Ref;

    fn next(&mut self) -> Option<Ref> {

        if !self.base.it.is_some() {
            self.base.start();
        }

        if !self.base.paths {
            return self.do_val()
        } else {
            return self.do_path()
        }
    }
}