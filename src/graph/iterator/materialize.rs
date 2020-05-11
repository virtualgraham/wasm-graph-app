use super::{Shape, Scanner, Index, Costs, Base, is_null, ShapeType};
use super::super::refs;
use super::super::value::Value;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;

pub const MATERIALIZE_LIMIT: i32 = 1000;

pub struct Materialize {
    sub: Rc<RefCell<dyn Shape>>,
    expected_size: i64
}

impl Materialize {
    pub fn new(sub: Rc<RefCell<dyn Shape>>) -> Rc<RefCell<Materialize>> {
        Rc::new(RefCell::new(Materialize {
            sub,
            expected_size: 0
        }))
    }

    pub fn new_with_size(sub: Rc<RefCell<dyn Shape>>, size: i64) -> Rc<RefCell<Materialize>> {
        Rc::new(RefCell::new(Materialize {
            sub,
            expected_size: size
        }))
    }
}


impl fmt::Display for Materialize {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Materialize")
    }
}


impl Shape for Materialize {
    fn iterate(&self) -> Rc<RefCell<dyn Scanner>> {
        return MaterializeNext::new(self.sub.clone())
    }

    fn lookup(&self) -> Rc<RefCell<dyn Index>> {
        return MaterializeContains::new(self.sub.clone())
    }

    fn stats(&mut self) -> Result<Costs, String> {
        let overhead = 2i64;
        let subit_stats = self.sub.borrow_mut().stats()?;
        let size;
        if self.expected_size > 0 {
            size = refs::Size{value: self.expected_size, exact: false};
        } else {
            size = subit_stats.size;
        }
        return Ok(Costs {
            contains_cost: overhead * subit_stats.next_cost,
            next_cost: overhead * subit_stats.next_cost,
            size: size
        })
    }

    fn optimize(&mut self) -> Option<Rc<RefCell<dyn Shape>>> {
        let n = self.sub.borrow_mut().optimize();
        if n.is_some() {
            self.sub = n.unwrap();
            if is_null(&mut*self.sub.borrow_mut()) {
                return Some(self.sub.clone())
            }
        }
        return None
    }

    fn sub_iterators(&self) -> Option<Vec<Rc<RefCell<dyn Shape>>>> {
        return Some(vec![self.sub.clone()])
    }

    fn shape_type(&mut self) -> ShapeType {
        ShapeType::Materialize
    }
}

#[derive(Debug, Clone)]
pub struct MaterializeResult {
    pub id: refs::Ref,
    pub tags: HashMap<String, refs::Ref>
}

struct MaterializeNext {
    sub: Rc<RefCell<dyn Shape>>,
    next: Rc<RefCell<dyn Scanner>>,
    contains_map: HashMap<Value, usize>,
    values: Vec<Vec<MaterializeResult>>,
    index: Option<usize>,
    sub_index: Option<usize>,
    has_run: bool,
    aborted: bool,
    err: Option<String>
}

impl MaterializeNext {
    fn new(sub: Rc<RefCell<dyn Shape>>) -> Rc<RefCell<MaterializeNext>> {
        // TODO: fix indexes being Options is not a good pattern
        // TODO: a vector of vectors is not a good pattern
        Rc::new(RefCell::new(MaterializeNext {
            sub: sub.clone(),
            next: sub.borrow().iterate(),
            contains_map: HashMap::new(),
            values: Vec::new(),
            index: None,
            sub_index: None,
            has_run: false,
            aborted: false,
            err: None
        }))
    }

    fn materialize_set(&mut self) {
        let mut i = 0;
        while self.next.borrow_mut().next() {
            i += 1;
            if i > MATERIALIZE_LIMIT {
                self.aborted = true;
                break
            }
            let id = self.next.borrow().result().unwrap();
            let val = id.key();

            if let Some(v) = val {
                if !self.contains_map.contains_key(v) {
                    self.contains_map.insert(v.clone(), self.values.len());
                    self.values.push(Vec::new());
                }
                let index = self.contains_map.get(&v);

                let mut tags: HashMap<String, refs::Ref> = HashMap::new();
                self.next.as_ref().borrow().tag_results(&mut tags);
                self.values[*index.unwrap()].push(MaterializeResult{id: id.clone(), tags});
                while self.next.borrow_mut().next_path() {
                    i += 1;
                    if i > MATERIALIZE_LIMIT {
                        self.aborted = true;
                        break
                    }
                    let mut tags: HashMap<String, refs::Ref> = HashMap::new();
                    self.next.as_ref().borrow().tag_results(&mut tags);
                    self.values[*index.unwrap()].push(MaterializeResult{id: id.clone(), tags});
                }
            }
        }
        self.err = self.next.borrow().err();
        if self.err.is_none() && self.aborted {
            // TODO: logging
            self.values = Vec::new();
            self.contains_map = HashMap::new();
            let _ = self.next.borrow_mut().close();
            self.next = self.sub.borrow().iterate();
        }
        self.has_run = true;
    }
}

impl fmt::Display for MaterializeNext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MaterializeNext")
    }
}

impl Base for MaterializeNext {

    fn tag_results(&self, dst: &mut HashMap<String, refs::Ref>) {
        if self.has_run {
            return
        }
        if self.aborted {
            self.next.borrow().tag_results(dst);
        }
        if self.result().is_none() {
            return
        }
    
        let x = self.index.unwrap();
        let y = self.sub_index.unwrap();

        let a = self.values.get(x).unwrap();
        let b = a.get(y).unwrap();

        for (tag, value) in &b.tags {
            dst.insert(tag.clone(), value.clone());
        }
     }

    fn result(&self) -> Option<refs::Ref> {
        if self.aborted {
            return self.next.borrow().result()
        }
        if self.values.is_empty() {
            return None
        }
        if self.index.is_none() {
            return None
        }
        if self.index.unwrap() >= self.values.len() {
            return None
        }

        let x = self.index.unwrap();
        let y = self.sub_index.unwrap();

        let a = self.values.get(x).unwrap();
        let b = a.get(y).unwrap();

        return Some(b.id.clone())
    }

    fn next_path(&mut self) -> bool {
        if !self.has_run {
            self.materialize_set();
        }
        if self.err.is_some() {
            return false
        }
        if self.aborted {
            return self.next.borrow_mut().next_path()
        }

        self.sub_index = Some(self.sub_index.unwrap() + 1);
        let v = self.values.get(self.index.unwrap()).unwrap();
        if self.sub_index.unwrap() >= v.len() {
            self.sub_index = Some(self.sub_index.unwrap() - 1);
            return false
        }
        return true
    }

    fn err(&self) -> Option<String> {
        return self.err.clone();
    }

    fn close(&mut self) -> Result<(), String> {
        self.values = Vec::new();
        self.contains_map = HashMap::new();
        self.has_run = false;
        self.next.borrow_mut().close()
    }
}

impl Scanner for MaterializeNext {
    fn next(&mut self) -> bool {
        if !self.has_run {
            self.materialize_set();
        }
        if self.err.is_some() {
            return false
        }
        if self.aborted {
            let n = self.next.borrow_mut().next();
            self.err = self.next.borrow().err();
            return n
        }
        self.index = Some(self.index.unwrap() + 1);
        self.sub_index = Some(0);
        if self.index.unwrap() >= self.values.len() {
            return false
        }
        return true
    }
}

struct MaterializeContains {
    next: Rc<RefCell<MaterializeNext>>,
    sub: Option<Rc<RefCell<dyn Index>>>,
}

impl MaterializeContains {
    fn new(sub: Rc<RefCell<dyn Shape>>) -> Rc<RefCell<MaterializeContains>>  {
        Rc::new(RefCell::new(MaterializeContains {
            next: MaterializeNext::new(sub),
            sub: None
        }))
    }

    fn run(&mut self) {
        self.next.borrow_mut().materialize_set();
        if self.next.borrow().aborted {
            self.sub = Some(self.next.borrow().sub.borrow().lookup());
        }
    }
}

impl fmt::Display for MaterializeContains {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MaterializeContains")
    }
}

impl Base for MaterializeContains {
    
    fn tag_results(&self, dst: &mut HashMap<String, refs::Ref>) { 
        if self.sub.is_some() {
            self.sub.as_ref().unwrap().borrow().tag_results(dst);
            return
        }
        self.next.borrow_mut().tag_results(dst)
    }

    fn result(&self) -> Option<refs::Ref> {
        if self.sub.is_some() {
            return self.sub.as_ref().unwrap().borrow().result();
        }
        self.next.borrow().result()
    }

    fn next_path(&mut self) -> bool {
        if !self.next.borrow().has_run {
            self.run();
        }
        if self.next.borrow().err().is_some() {
            return false
        }
        if self.sub.is_some() {
            return self.sub.as_ref().unwrap().borrow_mut().next_path();
        }

        return self.next.borrow_mut().next_path();
    }

    fn err(&self) -> Option<String> {
        let err = self.next.borrow().err();
        if err.is_some() {
            return err;
        } else if self.sub.is_none() {
            return None
        }
        return self.sub.as_ref().unwrap().borrow().err();
    }

    fn close(&mut self) -> Result<(), String> {
        let res = self.next.borrow_mut().close();
        if self.sub.is_some() {
            let res2 = self.sub.as_ref().unwrap().borrow_mut().close();
            if res2.is_err() && res.is_ok() {
                return res2
            }
        }
        res
    }
}

impl Index for MaterializeContains {
    fn contains(&mut self, v:&refs::Ref) -> bool {
        if !self.next.borrow().has_run {
            self.run();
        }

        if self.next.borrow().err().is_some() {
            return false
        }

        if self.sub.is_some() {
            return self.sub.as_ref().unwrap().borrow_mut().contains(v)
        }

        let i = if let Some(k) = v.key() {
            self.next.borrow_mut().contains_map.get(k).map(|x| x.clone())
        } else {
            None
        };

        if i.is_some() {
            self.next.borrow_mut().index = i;
            self.next.borrow_mut().sub_index = Some(0);
            return true;
        }

        false
    }
}