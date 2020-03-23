use std::rc::Rc;
use std::cell::RefCell;
use io_context::Context;
use super::{Shape, Scanner};
use super::refs::Ref;
use super::super::value::Value;
use super::super::quad::QuadStore;
use std::collections::HashMap;

pub struct Chain {
    ctx: Rc<RefCell<Context>>,
    s: Rc<RefCell<dyn Shape>>,
    it: Option<Rc<RefCell<dyn Scanner>>>,
    paths: bool,
    qs: Rc<RefCell<dyn QuadStore>>,
    optimize: bool, 
    limit: i64,
    n: i64
}


impl Chain {
    pub fn new(ctx: Rc<RefCell<Context>>, it: Rc<RefCell<dyn Shape>>, qs: Rc<RefCell<dyn QuadStore>>, optimize: bool, limit: i64, paths: bool) -> Chain {
        Chain {
            ctx,
            s: it,
            it: None,
            paths,
            qs,
            optimize,
            limit,
            n: 0
        }
    }

    fn quad_value_to_native() {

    }

    fn tags_to_value_map(&self, m: &HashMap<String, Ref>) -> Option<HashMap<String, Value>> {
        let mut output_map = HashMap::new();

        for (key, value) in m {
            match self.qs.borrow().name_of(value) {
                Some(v) => { output_map.insert(key.clone(), v); },
                None => {}
            };
        }
        
        if output_map.is_empty() {
            return None
        }

        return Some(output_map)
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
        i.close().unwrap();
    }


    fn next_val(&mut self) -> bool {
        let ok = (self.limit < 0 || self.n < self.limit) && self.it.as_ref().unwrap().borrow_mut().next(&*self.ctx.borrow());
        if ok {
            self.n += 1;
        }
        return ok
    }

    fn next_path(&mut self) -> bool {
        let ok = (self.limit < 0 || self.n < self.limit) && self.it.as_ref().unwrap().borrow_mut().next_path(&*self.ctx.borrow());
        if ok {
            self.n += 1;
        }
        return ok
    }

    fn do_val(&mut self) -> Option<HashMap<String, Value>> {

        if self.next_val() {
            self.paths = true;
            let mut tags = HashMap::new();
            self.it.as_ref().unwrap().borrow().tag_results(&mut tags);
            return self.tags_to_value_map(&tags)
        } else {
            self.end();
            return None
        }
    }

    fn do_path(&mut self)  -> Option<HashMap<String, Value>> {

        if self.next_path() {
            let mut tags = HashMap::new();
            self.it.as_ref().unwrap().borrow().tag_results(&mut tags);
            return self.tags_to_value_map(&tags)
        } else {
            self.paths = false;
            return self.do_val()
        }
    }
}

impl Iterator for Chain {
    type Item = HashMap<String, Value>;

    fn next(&mut self) -> Option<HashMap<String, Value>> {

        if !self.it.is_some() {
            self.start();
        }

        if !self.paths {
            return self.do_val()
        } else {
            return self.do_path()
        }
    }
}








    // pub fn tag_each(&mut self, callback: &mut dyn TagMapCallback) -> Result<(), String> {
    //     self.start();

    //     let mut mn = 0;

    //     while self.next() {
    //         if let Some(reason) = self.ctx.borrow().done() {
    //             return Err(reason.to_string());
    //         }
            
    //         let mut tags = HashMap::new();
    //         self.it.as_ref().unwrap().borrow().tag_results(&mut tags);
    //         let n = tags.len();
    //         if n > mn {
    //             mn = n;
    //         }
    //         callback.tag_map_callback(tags);

    //         while self.next_path() {
    //             if let Some(reason) = self.ctx.borrow().done() {
    //                 return Err(reason.to_string());
    //             }
                
    //             let mut tags = HashMap::new();
    //             self.it.as_ref().unwrap().borrow().tag_results(&mut tags);
    //             let n = tags.len();
    //             if n > mn {
    //                 mn = n;
    //             }
    //             callback.tag_map_callback(tags);
    //         }
    //     }

    //     self.end();

    //     match self.it.as_ref().unwrap().borrow().err() { Some(e) => Err(e), None => Ok(())}
    // }

    
