use super::value::Value;
use super::refs::{Size, Ref, Namer};
use super::iterator::{Shape};
use io_context::Context;
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub struct Quad {
    pub subject: Value,
    pub predicate: Value,
    pub object: Value,
    pub label: Value
}

#[derive(Debug, PartialEq, Clone)]
#[repr(C)]
pub enum Direction {
    Any,
    Subject,
    Predicate,
    Object,
    Label
}

impl Quad {
    pub fn new<V: Into<Value>>(subject:V, predicate:V, object:V, label:V) -> Quad {
        Quad {
            subject: subject.into(),
            predicate: predicate.into(),
            object: object.into(),
            label: label.into()
        }
    }

    pub fn get(&self, d: &Direction) -> &Value {
        match d {
            Direction::Subject => &self.subject,
            Direction::Predicate => &self.predicate,
            Direction::Object => &self.object,
            Direction::Label => &self.label,
        }
    }
}


impl fmt::Display for Quad {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} -- {} -> {}", self.subject, self.predicate, self.object)
        
    }
}


pub trait QuadStore : Namer {
    fn quad(&self, r: &Ref) -> Quad;
    fn quad_iterator(&self, d: &Direction, r: &Ref) -> Rc<RefCell<dyn Shape>>;
    fn quad_iterator_size(&self, ctx: &Context, d: &Direction, r: &Ref) -> Result<Size, String>;
    fn quad_direction(&self, r: &Ref, d: &Direction) -> Ref;
    fn stats(&self, ctx: &Context, exact: bool) -> Result<Stats, String>;
    
    fn apply_deltas(&self) -> Option<String>;
    fn new_quad_writer(&self) -> Result<String, String>;
    fn nodes_all_iterator(&self) -> Rc<RefCell<dyn Shape>>;
    fn quads_all_iterator(&self) -> Rc<RefCell<dyn Shape>>;
    fn close(&self) -> Option<String>;
}


pub struct Stats {
    pub nodes: Size,
    pub quads: Size
}