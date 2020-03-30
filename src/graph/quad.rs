use super::value::Value;
use super::refs::{Size, Ref, Namer};
use super::iterator::{Shape};
use super::transaction::Transaction;
use io_context::Context;
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;
use std::slice::Iter;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
#[derive(Serialize, Deserialize)]
pub struct Quad {
    pub subject: Value,
    pub predicate: Value,
    pub object: Value,
    pub label: Value
}


#[derive(Debug, PartialEq, Clone)]
#[repr(C)]
pub enum Direction {
    Subject,
    Predicate,
    Object,
    Label
}

impl Direction {
    pub fn iterator() -> Iter<'static, Direction> {
        static DIRECTIONS: [Direction; 4] = [Direction::Subject, Direction::Predicate, Direction::Object, Direction::Label];
        DIRECTIONS.iter()
    }

    pub fn to_byte(&self) -> i8 {
        match self {
            Direction::Subject => 1,
            Direction::Predicate => 2,
            Direction::Object => 3,
            Direction::Label => 4
        }
    } 
}

impl Quad {

    pub fn set_val(&mut self, dir: &Direction, v: Value) {
        match dir {
            Direction::Subject => self.subject = v,
            Direction::Predicate => self.predicate = v,
            Direction::Object => self.object = v,
            Direction::Label => self.label = v
        };
    }

    pub fn new_undefined_vals() -> Quad {
        Quad {
            subject: Value::None,
            predicate: Value::None,
            object: Value::None,
            label: Value::None
        }
    }

    pub fn new<W: Into<Value>, X: Into<Value>, Y: Into<Value>, Z: Into<Value>>(subject:W, predicate:X, object:Y, label:Z) -> Quad {
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
            Direction::Label => &self.label
        }
    }
}


impl fmt::Display for Quad {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} -- {} -> {}", self.subject, self.predicate, self.object)
        
    }
}

pub struct IgnoreOptions {
    pub ignore_dup: bool,
    pub ignore_missing: bool,
}

pub struct Delta {
    pub quad: Quad,
    pub action: Procedure
}

pub enum Procedure {
    Add,
    Delete
}

pub trait QuadStore : Namer {
    fn quad(&self, r: &Ref) -> Option<Quad>;
    fn quad_iterator(&self, d: &Direction, r: &Ref) -> Rc<RefCell<dyn Shape>>;
    fn quad_iterator_size(&self, ctx: &Context, d: &Direction, r: &Ref) -> Result<Size, String>;
    fn quad_direction(&self, r: &Ref, d: &Direction) -> Option<Ref>;
    fn stats(&self, ctx: &Context, exact: bool) -> Result<Stats, String>;
    
    fn apply_deltas(&mut self, deltas: Vec<Delta>, ignore_opts: &IgnoreOptions) -> Result<(), String>;
    // fn new_quad_writer(&self) -> Result<QuadWriter, String>;
    fn nodes_all_iterator(&self) -> Rc<RefCell<dyn Shape>>;
    fn quads_all_iterator(&self) -> Rc<RefCell<dyn Shape>>;
    fn close(&self) -> Option<String>;
}

pub struct QuadWriter {
    qs: Rc<RefCell<dyn QuadStore>>,
    ignore_opts: IgnoreOptions
}

impl QuadWriter {

    pub fn new(qs: Rc<RefCell<dyn QuadStore>>, ignore_opts: IgnoreOptions) -> QuadWriter {
        QuadWriter {
            qs,
            ignore_opts
        }
    }

    pub fn add_quad(&self, quad: Quad) -> Result<(), String> {
        self.qs.borrow_mut().apply_deltas(vec![Delta{action: Procedure::Add, quad}], &self.ignore_opts)
    }
    
    pub fn add_quad_set(&self, quads: Vec<Quad>) -> Result<(), String> {
        Ok(())
    }

    pub fn remove_quad(&self, quad: Quad) -> Result<(), String> {
        Ok(())
    }

    pub fn apply_transaction(&self, transaction: Transaction) -> Result<(), String> {
        Ok(())
    }

    pub fn remove_node(&self, value: Value) -> Result<(), String> {
        Ok(())
    }

    pub fn close(&self) -> Result<(), String> {
        Ok(())
    }
}


pub struct Stats {
    pub nodes: Size,
    pub quads: Size
}