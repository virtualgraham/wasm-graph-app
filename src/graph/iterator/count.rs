use super::{Shape, Base, Index, Scanner, Costs, ShapeType};
use super::super::refs;
use super::super::value::Value;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use io_context::Context;
use std::fmt;

pub struct Count {
    it: Rc<RefCell<dyn Shape>>,
    qs: Option<Rc<dyn refs::Namer>>
}

impl Count {
    pub fn new(it: Rc<RefCell<dyn Shape>>, qs: Option<Rc<dyn refs::Namer>>) -> Rc<RefCell<Count>> {
        Rc::new(RefCell::new(Count {
            it, 
            qs
        }))
    }
}


impl fmt::Display for Count {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Count")
    }
}

impl Shape for Count {
    fn iterate(&self) -> Rc<RefCell<dyn Scanner>> {
        CountNext::new(self.it.clone())
    }

    fn lookup(&self) -> Rc<RefCell<dyn Index>> {
        CountContains::new(self.it.clone(), self.qs.clone())
    }

    fn stats(&mut self, ctx: &Context) -> Result<Costs, String> {
       let mut stats = Costs {
           next_cost: 1,
           contains_cost: 0,
           size: refs::Size {
               value: 1,
               exact: true
           }
       };
       let sub = self.it.borrow_mut().stats(ctx);
       if sub.is_ok() && !sub.as_ref().unwrap().size.exact {
            stats.next_cost = sub.as_ref().unwrap().next_cost * sub.as_ref().unwrap().size.value;
       }
       Ok(stats)
    }

    fn optimize(&mut self, ctx: &Context) -> Option<Rc<RefCell<dyn Shape>>> {
        let optimized = self.it.borrow_mut().optimize(ctx);
        if optimized.is_some() { self.it = optimized.unwrap(); }
        return None
    }

    fn sub_iterators(&self) -> Option<Vec<Rc<RefCell<dyn Shape>>>> {
        return Some(vec![self.it.clone()])
    }

    fn shape_type(&mut self) -> ShapeType {
        ShapeType::Count
    }
}


struct CountNext {
    it: Rc<RefCell<dyn Shape>>,
    done: bool,
    result: Option<Value>,
    err: Option<String>
}

impl CountNext {
    fn new(it: Rc<RefCell<dyn Shape>>) -> Rc<RefCell<CountNext>> {
        Rc::new(RefCell::new(CountNext {
            it,
            done: false,
            result: None,
            err: None
        }))
    }
}


impl fmt::Display for CountNext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CountNext")
    }
}


impl Base for CountNext {
    #[allow(unused)]
    fn tag_results(&self, tags: &mut HashMap<String, refs::Ref>) {}

    fn result(&self) -> Option<refs::Ref> {
        if self.result.is_none() {
            return None
        }
        return Some(refs::pre_fetched(self.result.as_ref().unwrap().clone()))
    }
    
    #[allow(unused)]
    fn next_path(&mut self, ctx: &Context) -> bool {
        return false
    }
    
    fn err(&self) -> Option<String> {
        return None
    }
    
    fn close(&mut self) -> Result<(), String> {
        return Ok(())
    }
}

impl Scanner for CountNext {
    fn next(&mut self, ctx: &Context) -> bool {
        if self.done {
            return false
        }
        let st = self.it.borrow_mut().stats(ctx);
        if let Err(e) = st {
            self.err = Some(e);
            return false
        }
        let mut st = st.unwrap();

        if !st.size.exact {
            let sit = self.it.borrow().iterate();
            st.size.value = 0;
            while sit.borrow_mut().next(ctx) {
                st.size.value += 1;
                while sit.borrow_mut().next_path(ctx) {
                    st.size.value += 1;
                }
            }
            self.err = sit.borrow().err();
            let _ = sit.borrow_mut().close();
        }
        self.result = Some(Value::from(st.size.value));
        self.done = true;
        true
    }
}

struct CountContains {
    it: Rc<RefCell<CountNext>>,
    qs: Option<Rc<dyn refs::Namer>>
}

impl CountContains {
    fn new(it: Rc<RefCell<dyn Shape>>, qs: Option<Rc<dyn refs::Namer>>) -> Rc<RefCell<CountContains>> {
        Rc::new(RefCell::new(CountContains {
            it: CountNext::new(it),
            qs
        }))
    }
}

impl fmt::Display for CountContains {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CountContains")
    }
}

impl Base for CountContains {

    #[allow(unused)]
    fn tag_results(&self, tags: &mut HashMap<String, refs::Ref>) {}

    fn result(&self) -> Option<refs::Ref> {
        self.it.borrow().result()
    }

    #[allow(unused)]
    fn next_path(&mut self, ctx: &Context) -> bool {
        false
    }

    fn err(&self) -> Option<String> {
        self.it.borrow().err()
    }

    fn close(&mut self) -> Result<(), String> {
        self.it.borrow_mut().close()
    }
}

impl Index for CountContains {
    fn contains(&mut self, ctx: &Context, v:&refs::Ref) -> bool {
        if !self.it.borrow().done {
            self.it.borrow_mut().next(ctx);
        }
        if v.has_value() {
            return v.content == self.it.borrow().result
        }
        if self.qs.is_some() {
            return self.qs.as_ref().unwrap().name_of(v) == self.it.borrow().result
        }
        false
    }
}