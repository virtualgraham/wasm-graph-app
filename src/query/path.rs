use crate::graph::value::Value;
use crate::graph::quad::{Direction, QuadStore};
//use crate::graph::iterator::Shape;
use crate::query::shape::{Shape, AllNodes, Lookup};
use std::rc::Rc;
use std::cell::RefCell;
use super::morphism_apply_functions;




pub trait Morphism {
    fn reversal(&self, ctx: &PathContext) -> (Rc<dyn Morphism>, Option<PathContext>);
    fn apply(&self, shape: Rc<RefCell<dyn Shape>>, ctx: PathContext) -> (Rc<RefCell<dyn Shape>>, PathContext);
    fn is_tag(&self) -> bool { false }
    fn tags(&self) -> Option<Vec<String>> { None }
}



#[derive(Clone)]
pub struct PathContext {
    pub label_set: Rc<RefCell<dyn Shape>>
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
        np.stack.push(morphism_apply_functions::InMorphism::new(tags, via));
        return np
    }

    pub fn out_with_tags(self, tags: Vec<String>, via: Via) -> Path {
        let mut np = self.clone();
        np.stack.push(morphism_apply_functions::OutMorphism::new(tags, via));
        return np
    }


    pub fn both_with_tags(self, tags: Vec<String>, via: Via) -> Path {
        let mut np = self.clone();
        np.stack.push(morphism_apply_functions::BothMorphism::new(tags, via));
        return np
    }

    pub fn follow(self, path: Path) -> Path {
        let mut np = self.clone();
        np.stack.push(morphism_apply_functions::FollowMorphism::new(path));
        return np
    }

    pub fn follow_reverse(self, path: Path) -> Path {
        let mut np = self.clone();
        np.stack.push(morphism_apply_functions::FollowMorphism::new(path.reverse()));
        return np
    }

    pub fn reverse(self) -> Path {
        let new_path = Path::new(self.qs.clone(), Vec::new());
        let ctx = new_path.base_context.unwrap();
        for x in self.stack.iter().rev() {
            let (rev_morphism, ctx) = x.reversal(&ctx);
            new_path.stack.push(rev_morphism);
        }
        new_path
    }


    pub fn shape(self) -> Rc<RefCell<dyn Shape>> {
        return self.shape_from(Rc::new(RefCell::new(AllNodes())))
    }

    pub fn shape_from(self, from: Rc<RefCell<dyn Shape>>) -> Rc<RefCell<dyn Shape>> {
        let mut s = from;
        let mut ctx = self.base_context.unwrap();

        for m in self.stack {
            let r = m.apply(s, ctx);
            s = r.0;
            ctx = r.1;
        }

        s
    }
}



#[derive(Clone)]
pub enum Via {
    None,
    Values(Vec<Value>),
    Path(Path),
}

impl Via {
    pub fn as_shape(self) -> Rc<RefCell<dyn Shape>> {
        return match self {
            Via::None => Rc::new(RefCell::new(AllNodes())),
            Via::Path(path) => path.shape(),
            Via::Values(values) => Rc::new(RefCell::new(Lookup(values)))
        };
    }
}