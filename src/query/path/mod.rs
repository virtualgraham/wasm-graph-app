use crate::graph::value::Value;
use crate::graph::quad::{Direction, QuadStore};
//use crate::graph::iterator::Shape;
use crate::query::shape::Shape;
use std::rc::Rc;
use std::cell::RefCell;

mod morphism_apply_functions;


pub trait Morphism {
    fn reversal(&self, ctx: &PathContext) -> (Rc<dyn Morphism>, Option<PathContext>);
    fn apply(&self, shape: Rc<RefCell<dyn Shape>>, ctx: PathContext) -> (Rc<RefCell<dyn Shape>>, PathContext);
    fn is_tag(&self) -> bool { false }
    fn tags(&self) -> Option<Vec<String>> { None }
}



#[derive(Clone)]
pub struct PathContext {
    label_set: Rc<RefCell<dyn Shape>>
}


#[derive(Clone)]
pub struct Path {
    stack: Vec<Rc<dyn Morphism>>,
    qs: Option<Rc<RefCell<dyn QuadStore>>>,
    base_context: Option<PathContext>
}


impl Path {
    pub fn start_morphism(nodes: Vec<Value>) -> Path {
        Path::start_path(None, nodes)
    }

    pub fn start_path(qs: Option<Rc<RefCell<dyn QuadStore>>>, nodes: Vec<Value>) -> Path {
        Path::new(qs, vec![morphism_apply_functions::IsMorphism::new(nodes)])
    }

    pub fn new(qs: Option<Rc<RefCell<dyn QuadStore>>>, stack: Vec<Rc<dyn Morphism>>) -> Path {
        Path {
            stack,
            qs,
            base_context: None
        }   
    }

    pub fn is(self, nodes: Vec<Value>) -> Path {
        let mut np = self.clone();
        np.stack.push(morphism_apply_functions::IsMorphism::new(nodes));
        return np
    }

    pub fn in_with_tags(self, tags: Vec<String>, via: Via) -> Path {
        let mut np = self.clone();
        np.stack.push(morphism_apply_functions::InMorphism::new(nodes));
        return np
    }

    pub fn out_with_tags(self, tags: Vec<String>, via: Via) -> Path {
        let mut np = self.clone();
        np.stack.push(morphism_apply_functions::OutMorphism::new(nodes));
        return np
    }

}


pub fn new_in_out(from:Rc<RefCell<dyn Shape>>, via:Option<Rc<RefCell<dyn Shape>>>, labels:Option<Rc<RefCell<dyn Shape>>>, tags:Vec<String>, r#in: bool) -> Rc<RefCell<dyn Shape>> {
    let start = if r#in { Direction::Subject } else { Direction::Object };
    let goal = if r#in { Direction::Object } else { Direction::Subject };

    todo;
}

pub fn new_in(from:Rc<RefCell<dyn Shape>>, via:Option<Rc<RefCell<dyn Shape>>>, labels:Option<Rc<RefCell<dyn Shape>>>, tags:Vec<String>) -> Rc<RefCell<dyn Shape>> {
    new_in_out(from, via, labels, tags, true)
}

pub fn new_out(from:Rc<RefCell<dyn Shape>>, via:Option<Rc<RefCell<dyn Shape>>>, labels:Option<Rc<RefCell<dyn Shape>>>, tags:Vec<String>) -> Rc<RefCell<dyn Shape>> {
    new_in_out(from, via, labels, tags, false)
}


#[derive(Clone)]
pub enum Via {
    None,
    Values(Vec<Value>),
    Path(Path),
}

impl Via {
    fn as_shape(via: Vec<Via>) {
        
    }
}