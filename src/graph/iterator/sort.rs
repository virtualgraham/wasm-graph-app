use super::{Shape, ShapeType, Base, Index, Scanner, Costs};
use super::materialize::MaterializeResult;
use super::super::refs;
use super::super::quad::QuadStore;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use io_context::Context;
use std::cmp::Ordering;
use std::fmt;

pub struct Sort {
    qs: Rc<RefCell<dyn QuadStore>>,
    sub_it: Rc<RefCell<dyn Shape>>,
}

impl Sort {
    pub fn new(qs: Rc<RefCell<dyn QuadStore>>, sub_it: Rc<RefCell<dyn Shape>>) -> Rc<RefCell<Sort>> {
        Rc::new(RefCell::new(Sort {
            qs,
            sub_it
        }))
    }
}


impl fmt::Display for Sort {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Sort")
    }
}


impl Shape for Sort {
    fn iterate(&self) -> Rc<RefCell<dyn Scanner>> {
        SortNext::new(self.qs.clone(), self.sub_it.borrow().iterate())
    }

    fn lookup(&self) -> Rc<RefCell<dyn Index>> {
        self.sub_it.borrow().lookup()
    }

    fn stats(&mut self, ctx: &Context) -> Result<Costs, String> {
        let sub_stats = self.sub_it.borrow_mut().stats(ctx)?;
        return Ok(Costs {
            next_cost: sub_stats.next_cost * 2,
            contains_cost: sub_stats.contains_cost,
            size: refs::Size {
                value: sub_stats.size.value,
                exact: true
            }
        })
    }

    fn optimize(&mut self, ctx: &Context) -> Option<Rc<RefCell<dyn Shape>>> {
        let new_it = self.sub_it.borrow_mut().optimize(ctx);
        if new_it.is_some() {
            self.sub_it = new_it.unwrap()
        }
        None
    }

    fn sub_iterators(&self) -> Option<Vec<Rc<RefCell<dyn Shape>>>> {
        Some(vec![self.sub_it.clone()])
    }

    fn shape_type(&mut self) -> ShapeType {
        ShapeType::Sort
    }
}


struct SortValue  {
    result: MaterializeResult,
    string: String,
    paths: Vec<MaterializeResult>
}

impl Ord for SortValue {
    fn cmp(&self, other: &Self) -> Ordering {
        self.string.cmp(&other.string)
    }
}

impl PartialOrd for SortValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for SortValue {
    fn eq(&self, other: &Self) -> bool {
        self.string == other.string
    }
}

impl Eq for SortValue {}



struct SortNext {
    qs: Rc<RefCell<dyn QuadStore>>,
    sub_it: Rc<RefCell<dyn Scanner>>,
    ordered: Option<Vec<SortValue>>,
    result: Option<MaterializeResult>,
    err: Option<String>,
    index: usize,
    path_index: i32
}

impl SortNext {
    fn new(qs: Rc<RefCell<dyn QuadStore>>, sub_it: Rc<RefCell<dyn Scanner>>) -> Rc<RefCell<SortNext>> {
       Rc::new(RefCell::new(SortNext {
           qs,
           sub_it,
           ordered: None,
           result: None,
           err: None,
           index: 0,
           path_index: -1
       }))
    }
}

impl fmt::Display for SortNext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SortNext")
    }
}

impl Base for SortNext {

    fn tag_results(&self, tags: &mut HashMap<String, refs::Ref>) {
        for (k, v) in &self.result.as_ref().unwrap().tags {
            tags.insert(k.clone(), v.clone());
        }
    }

    fn result(&self) -> Option<refs::Ref> {
        return if let Some(r) = &self.result { Some(r.id.clone()) } else { None }
    }

    #[allow(unused)]
    fn next_path(&mut self, ctx: &Context) -> bool {
        if self.index >= self.ordered.as_ref().unwrap().len() {
            return false
        }
        let r = self.ordered.as_ref().unwrap().get(self.index).unwrap();
        if (self.path_index+1) >= r.paths.len() as i32 {
            return false
        }
        self.path_index += 1;
        self.result = Some(r.paths.get(self.path_index as usize).unwrap().clone());
        return true
    }

    fn err(&self) -> Option<String> {
        return self.err.clone()
    }

    fn close(&mut self) -> Result<(), String> {
        self.ordered = None;
        self.sub_it.borrow_mut().close()
    }
}

impl Scanner for SortNext {
    fn next(&mut self, ctx: &Context) -> bool {
        if self.err.is_some() {
            return false
        }

        if self.ordered.is_none() {
            let v = get_sorted_values(ctx, &self.qs, &self.sub_it);
            if let Err(e) = v {
                self.err = Some(e);
                return false
            }  
            if let Ok(val) = v {
                self.ordered = Some(val);
            }
        }

        if self.index >= self.ordered.as_ref().unwrap().len() {
            return false
        }

        self.path_index = -1;
        self.result = Some(self.ordered.as_ref().unwrap().get(self.index).as_ref().unwrap().result.clone());
        self.index += 1;

        return true
    }
}

fn get_sorted_values(ctx: &Context, qs: &Rc<RefCell<dyn QuadStore>>, it: &Rc<RefCell<dyn Scanner>>) -> Result<Vec<SortValue>, String> {
    let mut v:Vec<SortValue> = Vec::new();

    while it.borrow_mut().next(ctx) {
        let id = it.borrow().result().unwrap();
        let name = qs.borrow().name_of(&id).unwrap();
        let string = name.to_string();
        let mut tags = HashMap::new();
        it.borrow().tag_results(&mut tags);
        let mut val = SortValue {
            result: MaterializeResult {
                id: id.clone(),
                tags
            },
            string,
            paths: Vec::new()
        };
        while it.borrow_mut().next_path(ctx) {
            tags = HashMap::new();
            it.borrow().tag_results(&mut tags);
            val.paths.push(MaterializeResult {
                id: id.clone(),
                tags
            });
        }
        v.push(val);
    }

    if it.borrow().err().is_some() {
        return Err(it.borrow().err().unwrap());
    }

    v.sort();
    return Ok(v)
} 