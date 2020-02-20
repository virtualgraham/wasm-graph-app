use super::{Shape, Base, Index, Scanner, Costs, Morphism, Null, ShapeType};
use super::fixed::Fixed;
use super::save::tag;
use super::super::refs;
use super::super::value::Value;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use io_context::Context;
use std::fmt;

struct SeenAt {
    depth: i32,
    tags: HashMap<String, refs::Ref>,
    val: Option<refs::Ref>
}

pub struct Recursive {
    sub_it: Rc<RefCell<dyn Shape>>,
    morphism: Rc<dyn Morphism>,
    max_depth: i32,
    depth_tags: Vec<String>
}

const DEFAULT_MAX_RECURSIVE_STEPS:i32 = 50;

impl Recursive {
    pub fn new(sub_it: Rc<RefCell<dyn Shape>>, morphism: Rc<dyn Morphism>, max_depth: i32) -> Rc<RefCell<Recursive>> {
        Rc::new(RefCell::new(Recursive {
            sub_it,
            morphism,
            max_depth: if max_depth == 0 { DEFAULT_MAX_RECURSIVE_STEPS } else { max_depth },
            depth_tags: Vec::new()
        }))
    }
}


impl fmt::Display for Recursive {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Recursive")
    }
}


impl Shape for Recursive {
    fn iterate(&self) -> Rc<RefCell<dyn Scanner>> {
        return RecursiveNext::new(self.sub_it.borrow().iterate(), self.morphism.clone(), self.max_depth, self.depth_tags.clone())
    }

    fn lookup(&self) -> Rc<RefCell<dyn Index>> {
        return RecursiveContains::new(RecursiveNext::new(self.sub_it.borrow().iterate(), self.morphism.clone(), self.max_depth, self.depth_tags.clone()))
    }

    fn stats(&mut self, ctx: &Context) -> Result<Costs, String> {
        let base = Fixed::new(Vec::new());
        base.borrow_mut().add(refs::Ref::new_i64_node(20));
       
        let fanoutit = self.morphism.morph(&(base as Rc<RefCell<dyn Shape>>));

        let fanoutit_stats = fanoutit.borrow_mut().stats(ctx)?;
        let subit_stats = self.sub_it.borrow_mut().stats(ctx)?;
        let size = ((subit_stats.size.value * subit_stats.size.value) as f64).powf(5f64) as i64;
        return Ok(Costs {
            next_cost: subit_stats.next_cost + fanoutit_stats.next_cost,
            contains_cost: (subit_stats.next_cost + fanoutit_stats.next_cost)*(size/10) + subit_stats.contains_cost,
            size: refs::Size {
                value: size,
                exact: false
            }
        })
    }

    fn optimize(&mut self, ctx: &Context) -> Option<Rc<RefCell<dyn Shape>>> {
        let new_it = self.sub_it.borrow_mut().optimize(ctx);
        if new_it.is_some() {
            self.sub_it = new_it.unwrap();
        }
        return None
    }

    fn sub_iterators(&self) -> Option<Vec<Rc<RefCell<dyn Shape>>>> {
        return Some(vec![self.sub_it.clone()])
    }

    fn shape_type(&mut self) -> ShapeType {
        ShapeType::Recursive
    }
}



struct RecursiveNext {
    sub_it: Rc<RefCell<dyn Scanner>>,
    result: SeenAt,
    err: Option<String>,

    morphism: Rc<dyn Morphism>,
    seen: HashMap<Value, SeenAt>,
    next_it: Rc<RefCell<dyn Scanner>>,
    depth: i32,
    max_depth: i32,
    path_map: HashMap<Value, Vec<HashMap<String, refs::Ref>>>,
    path_index: usize,
    contains_value: Option<refs::Ref>,
    depth_tags: Vec<String>,
    depth_cache: Vec<refs::Ref>,
    base_it: Rc<RefCell<dyn Shape>>
}

impl RecursiveNext {
    fn new(sub_it: Rc<RefCell<dyn Scanner>>, morphism: Rc<dyn Morphism>, max_depth: i32, depth_tags: Vec<String>) -> Rc<RefCell<RecursiveNext>> {
        Rc::new(RefCell::new(RecursiveNext {
            sub_it,
            result: SeenAt {
                depth: 0,
                tags: HashMap::new(),
                val: None
            },
            err: None,
            morphism,
            seen: HashMap::new(),
            next_it: Null::new(),
            depth: 0,
            max_depth,
            path_map: HashMap::new(),
            path_index: 0,
            contains_value: None,
            depth_tags,
            depth_cache: Vec::new(),
            base_it: Fixed::new(vec![])
        }))
    }

    fn get_base_value(&self, val: &refs::Ref) -> refs::Ref {
        let mut at = self.seen.get(&val.key).unwrap();
        while at.depth != 1 {
            if at.depth == 0 {
                panic!("seen chain is broken");
            }
            at = self.seen.get(&at.val.as_ref().unwrap().key).unwrap();
        }
        return at.val.as_ref().unwrap().clone()
    }
}

const RECURSEIVE_BASE_TAG:&str = "__base_recursive";


impl fmt::Display for RecursiveNext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "RecursiveNext")
    }
}


impl Base for RecursiveNext {

    fn tag_results(&self, tags: &mut HashMap<String, refs::Ref>) {
        for tag in &self.depth_tags {
           let p = refs::pre_fetched(Value::from(self.result.depth));
           tags.insert(tag.clone(), p);
        }
        
        if self.contains_value.is_some() {
            let paths = self.path_map.get(&self.contains_value.as_ref().unwrap().key);
            if paths.is_some() && !paths.unwrap().is_empty() {
                for (k, v) in &paths.unwrap()[self.path_index] {
                    tags.insert(k.clone(), v.clone());
                } 
            }
        }
        
        self.next_it.borrow().tag_results(tags);
        tags.remove(RECURSEIVE_BASE_TAG);
    }

    fn result(&self) -> Option<refs::Ref> {
        return self.result.val.clone()
    }

    #[allow(unused)]
    fn next_path(&mut self, ctx: &Context) -> bool {
        let a = self.path_index + 1;
        let b = self.path_map.get(&self.contains_value.as_ref().unwrap().key).unwrap().len();
        if a >= b {
            return false
        }
        self.path_index += 1;
        return true
    }

    fn err(&self) -> Option<String> {
        return self.err.clone()
    }

    fn close(&mut self) -> Result<(), String> {
        let res = self.sub_it.borrow_mut().close();
        if res.is_err() {
            return res;
        }
        let res = self.next_it.borrow_mut().close();
        if res.is_err() {
            return res;
        }
        self.seen = HashMap::new();
        if self.err.is_some() { Err(self.err.as_ref().unwrap().clone()) } else { Ok(()) }
    }
}

impl Scanner for RecursiveNext {
    fn next(&mut self, ctx: &Context) -> bool {
        self.path_index = 0;

        if self.depth == 0 {
            while self.sub_it.borrow_mut().next(ctx) {
                let res = self.sub_it.borrow().result().unwrap();
                self.depth_cache.push(self.sub_it.borrow().result().unwrap());
                let mut tags:HashMap<String, refs::Ref> = HashMap::new();
                self.sub_it.borrow().tag_results(&mut tags);

                if !self.path_map.contains_key(&res.key) {
                    self.path_map.insert(res.key.clone(), vec![tags]);
                } else {
                    self.path_map.get_mut(&res.key).unwrap().push(tags);
                }

                while self.sub_it.borrow_mut().next_path(ctx) {
                    let mut tags:HashMap<String, refs::Ref> = HashMap::new();
                    self.sub_it.borrow().tag_results(&mut tags);

                    if !self.path_map.contains_key(&res.key) {
                        self.path_map.insert(res.key.clone(), vec![tags]);
                    } else {
                        self.path_map.get_mut(&res.key).unwrap().push(tags);
                    }
                }
            }
        }

        loop {
            if !self.next_it.borrow_mut().next(ctx) {
                if self.max_depth > 0 && self.depth >= self.max_depth {
                    return false
                } else if self.depth_cache.is_empty() {
                    return false
                }
                self.depth += 1;
                self.base_it = Fixed::new(self.depth_cache.clone());
                self.depth_cache = Vec::new();
                let _ = self.next_it.borrow_mut().close();

                self.next_it = self.morphism.morph(&tag(&self.base_it, &RECURSEIVE_BASE_TAG)).borrow().iterate();

                continue
            }
            let val = self.next_it.borrow().result().unwrap();
            let mut results:HashMap<String, refs::Ref> = HashMap::new();
            self.next_it.borrow().tag_results(&mut results);
            if !self.seen.contains_key(&val.key) {

                let base = results.get(RECURSEIVE_BASE_TAG).unwrap().clone();
                results.remove(RECURSEIVE_BASE_TAG);
                
                self.seen.insert(val.key.clone(), SeenAt {
                    val: Some(base),
                    depth: self.depth,
                    tags: results
                });
                self.result.depth = self.depth;
                self.result.val = Some(val.clone());
                self.contains_value = Some(self.get_base_value(&val));
                self.depth_cache.push(val.clone());
                return true
            }
        }
    }
}



struct RecursiveContains {
    next: Rc<RefCell<RecursiveNext>>,
    tags: HashMap<String, refs::Ref>
}

impl RecursiveContains {
    fn new(next: Rc<RefCell<RecursiveNext>>) -> Rc<RefCell<RecursiveContains>> {
        Rc::new(RefCell::new(RecursiveContains {
           next,
           tags: HashMap::new()
        }))
    }
}

impl fmt::Display for RecursiveContains {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "RecursiveContains({})", self.next.borrow().to_string())
    }
}

impl Base for RecursiveContains {

    fn tag_results(&self, tags: &mut HashMap<String, refs::Ref>) {
        self.next.borrow().tag_results(tags);
        for (key, val) in &self.tags {
            tags.insert(key.clone(), val.clone());
        }
    }

    fn result(&self) -> Option<refs::Ref> {
        self.next.borrow().result()
    }

    fn next_path(&mut self, ctx: &Context) -> bool {
        self.next.borrow_mut().next_path(ctx)
    }

    fn err(&self) -> Option<String> {
        self.next.borrow().err()
    }

    fn close(&mut self) -> Result<(), String> {
        self.next.borrow_mut().close()
    }
}

impl Index for RecursiveContains {
    fn contains(&mut self, ctx: &Context, val:&refs::Ref) -> bool {
        self.next.borrow_mut().path_index = 0;

        let depth = self.next.borrow_mut().seen.get(&val.key).map(|x| x.depth);
        let tags = self.next.borrow_mut().seen.get(&val.key).map(|x| x.tags.clone());

        if depth.is_some() && tags.is_some() {
            let contains_value = Some(self.next.borrow().get_base_value(val));
            self.next.borrow_mut().contains_value = contains_value;
            self.next.borrow_mut().result.depth = depth.unwrap();
            self.next.borrow_mut().result.val = Some(val.clone());
            self.tags = tags.unwrap();
            return true
        }
        while self.next.borrow_mut().next(ctx) {
            if self.next.borrow().result().unwrap().key == val.key {
                return true
            }
        }
        return false
    }
}