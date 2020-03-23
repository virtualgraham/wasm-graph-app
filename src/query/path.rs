use crate::graph::value::Value;
use crate::graph::iterator;
use crate::graph::quad::{Direction, QuadStore};
//use crate::graph::iterator::Shape;
use crate::query::shape::{Shape, AllNodes, Lookup, IteratorShape, build_iterator, ValueFilter};
use std::rc::Rc;
use std::cell::RefCell;
use super::morphism_apply_functions;
use io_context::Context;
use super::gizmo::Session;


pub trait Morphism {
    fn reversal(&self, ctx: &PathContext) -> (Rc<dyn Morphism>, Option<PathContext>);
    fn apply(&self, shape: Rc<RefCell<dyn Shape>>, ctx: &PathContext) -> (Rc<RefCell<dyn Shape>>, Option<PathContext>);
    fn is_tag(&self) -> bool { false }
    fn tags(&self) -> Option<Vec<String>> { None }
}



#[derive(Clone)]
pub struct PathContext {
    pub label_set: Option<Rc<RefCell<dyn Shape>>>
}


#[derive(Clone)]
pub struct Path {
    stack: Vec<Rc<dyn Morphism>>,
    pub qs: Option<Rc<RefCell<dyn QuadStore>>>,
    pub base_context: PathContext
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
            base_context: PathContext{ label_set: None }
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
        println!("np.stack.len() {}", np.stack.len());
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

    pub fn follow_recursive(self, via: Via, max_depth: i32, tags: Vec<String>) -> Path {
        let mut np = self.clone();
        let path = match via {
            Via::Values(v) => Path::start_morphism(v),
            Via::Path(p) => p,
            Via::None => panic!("did not pass a predicate or a Path to FollowRecursive"),
        };
        np.stack.push(morphism_apply_functions::FollowRecursiveMorphism::new(path, max_depth, tags));
        return np
    }

    pub fn and(self, path: Path) -> Path {
        let mut np = self.clone();
        np.stack.push(morphism_apply_functions::AndMorphism::new(path));
        return np
    }
    
    pub fn or(self, path: Path) -> Path {
        let mut np = self.clone();
        np.stack.push(morphism_apply_functions::OrMorphism::new(path));
        return np
    }


    pub fn filters(self, filters: Vec<Rc<dyn ValueFilter>>)  -> Path {
        let mut np = self.clone();
        np.stack.push(morphism_apply_functions::FilterMorphism::new(filters));
        return np
    }


    pub fn reverse(self) -> Path {
        let mut new_path = Path::new(self.qs.clone(), Vec::new());
        let ctx = new_path.base_context.clone();

        for x in self.stack.iter().rev() {
            let (rev_morphism, _) = x.reversal(&ctx); 
            new_path.stack.push(rev_morphism);
        }
        
        new_path
    }


    pub fn shape(&self) -> Rc<RefCell<dyn Shape>> {
        return self.shape_from(Rc::new(RefCell::new(AllNodes())))
    }

    pub fn shape_from(&self, from: Rc<RefCell<dyn Shape>>) -> Rc<RefCell<dyn Shape>> {
        let mut s = from;
        let mut ctx = self.base_context.clone();

        for m in &self.stack {
            let r = m.apply(s, &ctx);
            s = r.0;
            ctx = match r.1 { Some(c) => c, None => ctx };
        }

        s
    }


    pub fn build_iterator_on(&self, ctx: &Context, qs: Rc<RefCell<dyn QuadStore>>) -> Rc<RefCell<dyn iterator::Shape>> {
        let s = self.shape().clone();
        build_iterator(ctx, qs, s)
    }
}



pub struct MorphismForPath {
    path: Path,
    qs: Rc<RefCell<dyn QuadStore>>
}

impl MorphismForPath {
    pub fn new(path: Path, qs: Rc<RefCell<dyn QuadStore>>) -> Rc<dyn iterator::Morphism> {
        return Rc::new(MorphismForPath {
            path,
            qs
        })
    }
}

impl iterator::Morphism for MorphismForPath {
    fn morph(&self, shape: Rc<RefCell<dyn iterator::Shape>>) -> Rc<RefCell<dyn iterator::Shape>> {
        return self.path.clone().shape_from(Rc::new(RefCell::new(IteratorShape{it: shape, sent: false}))).borrow().build_iterator(self.qs.clone())
    }
}


#[derive(Clone)]
pub enum Via {
    None,
    Values(Vec<Value>),
    Path(Path),
}

impl Via {
    pub fn as_shape(&self) -> Rc<RefCell<dyn Shape>> {
        return match self {
            Via::None => Rc::new(RefCell::new(AllNodes())),
            Via::Path(path) => path.clone().shape(),
            Via::Values(values) => Rc::new(RefCell::new(Lookup(values.clone())))
        };
    }
}

impl From<Option<Value>> for Via {
    fn from(v: Option<Value>) -> Self {
        match v {
            Some(v) => Via::Values(vec![v]),
            None => Via::None
        }
    }
}


impl From<String> for Via {
    fn from(v: String) -> Self {
        Via::Values(vec![v.into()])
    }
}

impl From<&str> for Via {
    fn from(v: &str) -> Self {
        Via::Values(vec![v.into()])
    }
}

impl From<Value> for Via {
    fn from(v: Value) -> Self {
        Via::Values(vec![v])
    }
}

impl From<Vec<Value>> for Via {
    fn from(v: Vec<Value>) -> Self {
        Via::Values(v)
    }
}

impl From<Path> for Via {
    fn from(p: Path) -> Self {
        Via::Path(p)
    }
}

