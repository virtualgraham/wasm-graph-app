pub mod and;
pub mod materialize;
pub mod count;
pub mod fixed;
pub mod limit;
pub mod not;
pub mod or;
pub mod recursive;
pub mod save;
pub mod resolver;
pub mod skip;
pub mod sort;
pub mod unique;
pub mod value_filter;
pub mod iterate;

use std::collections::HashMap;
use super::refs;
use std::rc::Rc;
use std::cell::RefCell;
use io_context::Context;
use std::fmt;

use super::iterator::fixed::Fixed;
use super::iterator::save::Save;

#[derive(Clone)]
pub struct Tags {
    pub tags: Vec<String>,
    pub fixed_tags: HashMap<String, refs::Ref>
}

impl Tags {
    pub fn add_tags(&mut self, tags: &Vec<String>) {
        self.tags.append(&mut tags.clone());
    }

    pub fn add_fixed_tag(&mut self, tag: String, value: &refs::Ref) {
        self.fixed_tags.insert(tag, value.clone());
    }

    pub fn copy_from(&mut self, st:&Tags) {
        self.add_tags(&st.tags);
        if st.fixed_tags.is_empty() {
            return
        }
        for (k, v) in &st.fixed_tags {
            self.fixed_tags.insert(k.clone(), v.clone());
        }
    }
}


pub trait Base : fmt::Display {
    fn tag_results(&self, tags: &mut HashMap<String, refs::Ref>);
    fn result(&self) -> Option<refs::Ref>;
    fn next_path(&mut self, ctx: &Context) -> bool;
    fn err(&self) -> Option<String>;
    fn close(&mut self) -> Result<(), String>;
}


pub trait Scanner : Base {
    fn next(&mut self, ctx: &Context) -> bool;
}


pub trait Index : Base {
    fn contains(&mut self, ctx: &Context, v:&refs::Ref) -> bool;
}


#[derive(Debug, Clone)]
pub struct Costs {
    pub contains_cost: i64,
    pub next_cost: i64,
    pub size: refs::Size
}

impl Costs {
    pub fn new() -> Costs {
        Costs {
            contains_cost: 0,
            next_cost: 0,
            size: refs::Size::new()
        }
    }
}


pub enum ShapeType<'a> {
    And,
    Count,
    Error,
    Fixed(&'a mut Fixed),
    HasA,
    Int64,
    Limit,
    LinksTo,
    Materialize,
    Not,
    Null,
    Or,
    Recursive,
    Resolver,
    Save(&'a mut Save),
    Skip,
    Sort,
    Test,
    Unique,
    ValueFilter,
    MemStoreIterator
}



pub trait Shape : fmt::Display {
    fn iterate(&self) -> Rc<RefCell<dyn Scanner>>;

    fn lookup(&self) -> Rc<RefCell<dyn Index>>;

    // self is mut so stats can be cached
    fn stats(&mut self, ctx: &Context) -> Result<Costs, String>;

    // Optimizes an iterator. Can replace the iterator, or merely move things
	// around internally. If it chooses to replace it with a better iterator,
	// returns Some with the new Shape if not, it returns None.
    fn optimize(&mut self, ctx: &Context) -> Option<Rc<RefCell<dyn Shape>>>;

    fn sub_iterators(&self) -> Option<Vec<Rc<RefCell<dyn Shape>>>>;

    fn shape_type(&mut self) -> ShapeType;
}




pub trait Morphism {
    fn morph(&self, shape: Rc<RefCell<dyn Shape>>) -> Rc<RefCell<dyn Shape>>;
}


pub fn is_null(it: &Rc<RefCell<dyn Shape>>) -> bool {
    if let ShapeType::Null = it.borrow_mut().shape_type() {
        return true
    } 
    return false
}


pub fn height(it: &Rc<RefCell<dyn Shape>>, filter: fn(&Rc<RefCell<dyn Shape>>) -> bool) -> i32 {
    if !filter(it) { return 1 }

    let subs = it.borrow().sub_iterators();
    let mut max_depth = 0;

    if subs.is_none() { return 1 }

    for sub in subs.unwrap() {
        let h = height(&sub, filter);
        if h > max_depth {
            max_depth = h;
        }
    }

    return max_depth + 1;
}

#[derive(Debug, Clone)]
pub struct Null {}

impl Null {
    pub fn new() -> Rc<RefCell<Null>> {
        Rc::new(RefCell::new(Null{}))
    }
}

impl fmt::Display for Null {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Null")
    }
}

impl Base for Null {
    #[allow(unused)]
    fn tag_results(&self, tags: &mut HashMap<String, refs::Ref>) { }

    fn result(&self) -> Option<refs::Ref> {
        None
    }

    #[allow(unused)]
    fn next_path(&mut self, ctx: &Context) -> bool {
        false
    }

    fn err(&self) -> Option<String> {
        None 
    }

    fn close(&mut self) -> Result<(), String> {
        Ok(())
    }
}


impl Scanner for Null {
    #[allow(unused)]
    fn next(&mut self, ctx: &Context) -> bool {
        false
    }
}


impl Index for Null {
    #[allow(unused)]
    fn contains(&mut self, ctx: &Context, v:&refs::Ref) -> bool {
        false
    }
}


impl Shape for Null {
    fn iterate(&self) -> Rc<RefCell<dyn Scanner>> {
        Rc::new(RefCell::new(self.clone()))
    }

    fn lookup(&self) -> Rc<RefCell<dyn Index>> {
        Rc::new(RefCell::new(self.clone()))
    }

    #[allow(unused)]
    fn stats(&mut self, ctx: &Context) -> Result<Costs, String> {
        return Ok(Costs::new())
    }

    #[allow(unused)]
    fn optimize(&mut self, ctx: &Context) -> Option<Rc<RefCell<dyn Shape>>> {
        None
    }

    fn sub_iterators(&self) -> Option<Vec<Rc<RefCell<dyn Shape>>>> {
        None
    }

    fn shape_type(&mut self) -> ShapeType {
        ShapeType::Null
    }
}


#[derive(Debug, Clone)]
pub struct Error {
    err: String
}


impl Error {
    pub fn new(err: String) -> Rc<RefCell<Error>> {
        Rc::new(RefCell::new(Error{
            err
        }))
    }
}

impl Base for Error {

    #[allow(unused)]
    fn tag_results(&self, tags: &mut HashMap<String, refs::Ref>) { }

    fn result(&self) -> Option<refs::Ref> {
        None
    }

    #[allow(unused)]
    fn next_path(&mut self, ctx: &Context) -> bool {
        false
    }

    fn err(&self) -> Option<String> {
        Some(self.err.clone()) 
    }

    fn close(&mut self) -> Result<(), String> {
        Err(self.err.clone())
    }
}


impl Scanner for Error {
    #[allow(unused)]
    fn next(&mut self, ctx: &Context) -> bool {
        false
    }
}


impl Index for Error {
    #[allow(unused)]
    fn contains(&mut self, ctx: &Context, v:&refs::Ref) -> bool {
        false
    }
}


impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error({})", self.err)
    }
}


impl Shape for Error {
    fn iterate(&self) -> Rc<RefCell<dyn Scanner>> {
        Rc::new(RefCell::new(self.clone()))
    }

    fn lookup(&self) -> Rc<RefCell<dyn Index>>  {
        Rc::new(RefCell::new(self.clone()))
    }

    #[allow(unused)]
    fn stats(&mut self, ctx: &Context) -> Result<Costs, String> {
        return Ok(Costs::new())
    }

    #[allow(unused)]
    fn optimize(&mut self, ctx: &Context) -> Option<Rc<RefCell<dyn Shape>>> {
        None
    }

    fn sub_iterators(&self) -> Option<Vec<Rc<RefCell<dyn Shape>>>> {
        None
    }

    fn shape_type(&mut self) -> ShapeType {
        ShapeType::Error
    }
}