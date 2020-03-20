
use crate::graph::value::Value;
use std::cell::RefCell;
use std::rc::Rc;
use super::path::{Morphism, PathContext};
use crate::query::shape::{Shape, ShapeType, Lookup, Union, Null, Intersect, new_in_out};
use crate::query::path::{Via, Path};
use crate::graph::quad::{QuadStore};

fn join(its: Vec<Rc<RefCell<dyn Shape>>>) -> Rc<RefCell<dyn Shape>> {
    if its.is_empty() {
        return Null::new()
    } 

    if let ShapeType::AllNodes = its[0].borrow_mut().shape_type() {
        return join(its[1..].to_vec())
    }

    return Intersect::new(its)
}

//////////////////////////////////////////////////////////

pub struct IsMorphism {
    nodes: Vec<Value>
}

impl IsMorphism {
    pub fn new(nodes: Vec<Value>) -> Rc<dyn Morphism> {
        Rc::new(IsMorphism {
            nodes
        })
    }
}

impl Morphism for IsMorphism {
    fn reversal(&self, ctx: &PathContext) -> (Rc<dyn Morphism>, Option<PathContext>) {
        (IsMorphism::new(self.nodes.clone()), None)
    }

    fn apply(&self, shape: Rc<RefCell<dyn Shape>>, ctx: &PathContext) -> (Rc<RefCell<dyn Shape>>, Option<PathContext>) {
        if self.nodes.is_empty() {
            return (shape, None)
        }
        let s = Lookup::new(self.nodes.clone());
        if let ShapeType::AllNodes = shape.borrow_mut().shape_type() {

        }
        return (join(vec![s, shape]), None)
    }
}

//////////////////////////////////////////////////////////

pub struct InMorphism {
    tags: Vec<String>,
    via: Via
}

impl InMorphism {
    pub fn new(tags: Vec<String>, via: Via) -> Rc<dyn Morphism> {
        Rc::new(InMorphism {
            tags, 
            via
        })
    }
}

impl Morphism for InMorphism {
    fn reversal(&self, ctx: &PathContext) -> (Rc<dyn Morphism>, Option<PathContext>) {
        (OutMorphism::new(self.tags.clone(), self.via.clone()), None)
    }

    fn apply(&self, shape: Rc<RefCell<dyn Shape>>, ctx: &PathContext) -> (Rc<RefCell<dyn Shape>>, Option<PathContext>) {
        return (new_in_out(shape, self.via.as_shape(), Some(ctx.label_set.clone()), self.tags.clone(), true), None)
    }
}

//////////////////////////////////////////////////////////

pub struct OutMorphism {
    tags: Vec<String>,
    via: Via
}

impl OutMorphism {
    pub fn new(tags: Vec<String>, via: Via) -> Rc<dyn Morphism> {
        Rc::new(OutMorphism {
            tags, 
            via
        })
    }
}

impl Morphism for OutMorphism {
    fn reversal(&self, ctx: &PathContext) -> (Rc<dyn Morphism>, Option<PathContext>) {
        (InMorphism::new(self.tags.clone(), self.via.clone()), None)
    }

    fn apply(&self, shape: Rc<RefCell<dyn Shape>>, ctx: &PathContext) -> (Rc<RefCell<dyn Shape>>, Option<PathContext>) {
        return (new_in_out(shape, self.via.as_shape(), Some(ctx.label_set.clone()), self.tags.clone(), false), None)
    }
}

//////////////////////////////////////////////////////////

pub struct BothMorphism {
    tags: Vec<String>,
    via: Via
}

impl BothMorphism {
    pub fn new(tags: Vec<String>, via: Via) -> Rc<dyn Morphism> {
        Rc::new(BothMorphism {
            tags, 
            via
        })
    }
}

impl Morphism for BothMorphism {
    fn reversal(&self, ctx: &PathContext) -> (Rc<dyn Morphism>, Option<PathContext>) {
        (BothMorphism::new(self.tags.clone(), self.via.clone()), None)
    }

    fn apply(&self, shape: Rc<RefCell<dyn Shape>>, ctx: &PathContext) -> (Rc<RefCell<dyn Shape>>, Option<PathContext>) {
        let via = self.via.as_shape();
        return (Rc::new(RefCell::new(Union(vec![
            new_in_out(shape.clone(), via.clone(), Some(ctx.label_set.clone()), self.tags.clone(), true),
            new_in_out(shape.clone(), via.clone(), Some(ctx.label_set.clone()), self.tags.clone(), false)
        ]))), None)
    }
}

//////////////////////////////////////////////////////////

pub struct FollowMorphism {
    path: Path
}

impl FollowMorphism {
    pub fn new(path: Path) -> Rc<dyn Morphism> {
        Rc::new(FollowMorphism {
            path
        })
    }
}

impl Morphism for FollowMorphism {
    fn reversal(&self, ctx: &PathContext) -> (Rc<dyn Morphism>, Option<PathContext>) {
        (FollowMorphism::new(self.path.clone().reverse()), None)
    }

    fn apply(&self, shape: Rc<RefCell<dyn Shape>>, ctx: &PathContext) -> (Rc<RefCell<dyn Shape>>, Option<PathContext>) {
        (self.path.clone().shape_from(shape), None)
    }
}

//////////////////////////////////////////////////////////

pub struct FollowRecursiveMorphism {
    path: Path,
    max_depth: i32,
    tags: Vec<String>,

}

impl FollowRecursiveMorphism {
    pub fn new(path: Path, max_depth: i32, tags: Vec<String>) -> Rc<dyn Morphism> {
        Rc::new(FollowRecursiveMorphism {
            path,
            max_depth,
            tags
        })
    }
}

impl Morphism for FollowRecursiveMorphism {
    fn reversal(&self, ctx: &PathContext) -> (Rc<dyn Morphism>, Option<PathContext>) {
        (FollowRecursiveMorphism::new(self.path.clone().reverse(), self.max_depth, self.tags.clone()), None)
    }

    fn apply(&self, shape: Rc<RefCell<dyn Shape>>, ctx: &PathContext) -> (Rc<RefCell<dyn Shape>>, Option<PathContext>) {
        (self.path.clone().shape_from(shape), None)
    }
}

//////////////////////////////////////////////////////////

pub struct AndMorphism {
    path: Path
}

impl AndMorphism {
    pub fn new(path: Path) -> Rc<dyn Morphism> {
        Rc::new(AndMorphism {
            path
        })
    }
}

impl Morphism for AndMorphism {
    fn reversal(&self, ctx: &PathContext) -> (Rc<dyn Morphism>, Option<PathContext>) {
        (AndMorphism::new(self.path.clone()), None)
    }

    fn apply(&self, shape: Rc<RefCell<dyn Shape>>, ctx: &PathContext) -> (Rc<RefCell<dyn Shape>>, Option<PathContext>) {
        (join(vec![shape, self.path.clone().shape()]), None)
    }
}

//////////////////////////////////////////////////////////

pub struct OrMorphism {
    path: Path
}

impl OrMorphism {
    pub fn new(path: Path) -> Rc<dyn Morphism> {
        Rc::new(OrMorphism {
            path
        })
    }
}

impl Morphism for OrMorphism {
    fn reversal(&self, ctx: &PathContext) -> (Rc<dyn Morphism>, Option<PathContext>) {
        (OrMorphism::new(self.path.clone()), None)
    }

    fn apply(&self, shape: Rc<RefCell<dyn Shape>>, ctx: &PathContext) -> (Rc<RefCell<dyn Shape>>, Option<PathContext>) {
       (Rc::new(RefCell::new(Union(vec![shape, self.path.clone().shape()]))), None)
    }
}


//////////////////////////////////////////////////////////