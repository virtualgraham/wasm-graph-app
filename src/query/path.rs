use crate::graph::value::Value;
use crate::graph::iterator;
use crate::graph::quad::{Direction, QuadStore};
//use crate::graph::iterator::Shape;
use crate::query::shape::{Shape, AllNodes, Lookup, IteratorShape, build_iterator, ValueFilter};
use std::rc::Rc;
use std::cell::RefCell;
use super::morphism;
use io_context::Context;
use crate::query::gizmo;






#[derive(Clone)]
pub struct PathContext {
    pub label_set: Option<Rc<RefCell<dyn Shape>>>
}


#[derive(Clone)]
pub struct Path {
    stack: Vec<Rc<dyn morphism::Morphism>>,
    pub qs: Option<Rc<RefCell<dyn QuadStore>>>,
    pub base_context: PathContext
}


impl Path {
    pub fn start_morphism(nodes: Vec<Value>) -> Path {
        Path::start_path(None, nodes)
    }

    pub fn start_path(qs: Option<Rc<RefCell<dyn QuadStore>>>, nodes: Vec<Value>) -> Path {
        Path::new(qs, vec![morphism::IsMorphism::new(nodes)])
    }

    pub fn new(qs: Option<Rc<RefCell<dyn QuadStore>>>, stack: Vec<Rc<dyn morphism::Morphism>>) -> Path {
        Path {
            stack,
            qs,
            base_context: PathContext{ label_set: None }
        }   
    }

    ///////

    pub fn is(&mut self, nodes: Vec<Value>) {
        self.stack.push(morphism::IsMorphism::new(nodes));
    }

    pub fn in_with_tags(&mut self, tags: Vec<String>, via: Via) {
        self.stack.push(morphism::InMorphism::new(tags, via));
    }

    pub fn out_with_tags(&mut self, tags: Vec<String>, via: Via)  {
        self.stack.push(morphism::OutMorphism::new(tags, via));
    }


    pub fn both_with_tags(&mut self, tags: Vec<String>, via: Via)  {
        self .stack.push(morphism::BothMorphism::new(tags, via));
    }

    pub fn follow(&mut self, path: Path) {
        self.stack.push(morphism::FollowMorphism::new(path));
    }

    pub fn follow_reverse(&mut self, mut path: Path) {
        self.stack.push(morphism::FollowMorphism::new(path.reverse()));
    }

    pub fn follow_recursive(&mut self, via: Via, max_depth: i32, tags: Vec<String>) {
        let path = match via {
            Via::Values(v) => Path::start_morphism(v),
            Via::Path(p) => p,
            Via::None => panic!("did not pass a predicate or a Path to FollowRecursive"),
        };
        self.stack.push(morphism::FollowRecursiveMorphism::new(path, max_depth, tags));
    }

    pub fn and(&mut self, path: Path) {
        self.stack.push(morphism::AndMorphism::new(path));
    }
    
    pub fn or(&mut self, path: Path) {
        self.stack.push(morphism::OrMorphism::new(path));
    }


    pub fn filters(&mut self, filters: Vec<Rc<dyn ValueFilter>>) {
        self.stack.push(morphism::FilterMorphism::new(filters));
    }

    pub fn tag(&mut self, tags: Vec<String>) {
        self.stack.push(morphism::TagMorphism::new(tags));
    }

    pub fn back(&mut self, tag: String) -> Option<Path> {
        let mut new_path = Path::new(self.qs.clone(), Vec::new());
        let mut i = (self.stack.len() - 1) as i64;
        loop {
            println!("{}", i);
            if i < 0 {
                return Some(self.reverse())
            }
            if self.stack[i as usize].is_tag() {
                let tags = self.stack[i as usize].tags();
                if let Some(t) = tags {
                    for x in t {
                        if x == tag {
                            self.stack = self.stack[0..((i+1) as usize)].to_vec();
                            self.and(new_path);
                            return None
                        }
                    }
                }
            }
            let rev = self.stack[i as usize].reversal(&mut new_path.base_context);
            new_path.stack.push(rev.0);
            i -= 1;
        }
    }

    pub fn except(&mut self, path: Path) {
        self.stack.push(morphism::ExceptMorphism::new(path));
    }

    pub fn unique(&mut self) {
        self.stack.push(morphism::UniqueMorphism::new());
    }

    pub fn has(&mut self, via: Via, rev: bool, nodes: Vec<Value>) {
        self.stack.push(morphism::HasShapeMorphism::new_has_morphism(via, rev, nodes));
    }

    pub fn has_filter(&mut self, via: Via, rev: bool, nodes: Vec<Rc<dyn ValueFilter>>) {
        self.stack.push(morphism::HasShapeMorphism::new_has_filter_morphism(via, rev, nodes));
    }

    pub fn skip(&mut self, offset: i64) {
        self.stack.push(morphism::SkipMorphism::new(offset));
    }

    pub fn limit(&mut self, limit: i64) {
        self.stack.push(morphism::LimitMorphism::new(limit));
    }

    pub fn order(&mut self) {
        self.stack.push(morphism::OrderMorphism::new());
    }

    // pub fn count(&mut self) {
    //     self.stack.push(morphism::CountMorphism::new());
    // }

    pub fn save(&mut self, via: Via, tag: String, rev: bool, opt: bool) {
        self.stack.push(morphism::SaveMorphism::new(via, tag, rev, opt));
    }

    pub fn predicates(&mut self, rev: bool) {
        self.stack.push(morphism::PredicatesMorphism::new(rev));
    }

    pub fn save_predicates(&mut self, tag: String, rev: bool) {
        self.stack.push(morphism::SavePredicatesMorphism::new(tag, rev));
    }

    pub fn labels(&mut self) {
        self.stack.push(morphism::LabelsMorphism::new());
    }

    pub fn label_context_with_tags(&mut self, via: Via, tags: Vec<String>)  {
        self.stack.push(morphism::LabelContextMorphism::new(via, tags));
    }

    ///////
 

    pub fn reverse(&mut self) -> Path {
        let mut new_path = Path::new(self.qs.clone(), Vec::new());
        let mut ctx = new_path.base_context.clone();

        for x in self.stack.iter().rev() {
            let (rev_morphism, _) = x.reversal(&mut ctx); 
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
            let r = m.apply(s, &mut ctx);
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
            Via::Path(path) => path.shape(),
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

impl From<&gizmo::Path> for Via {
    fn from(p: &gizmo::Path) -> Self {
        Via::Path(p.clone().path)
    }
}

