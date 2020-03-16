
use crate::graph::value::Value;
use std::cell::RefCell;
use std::rc::Rc;
use super::path::{Morphism, PathContext};
use crate::query::shape::{Shape, ShapeType, Lookup, Union, Null, Intersect, new_in_out};
use crate::query::path::{Via, Path};
use crate::graph::quad::{QuadStore};

pub fn join(its: Vec<Rc<RefCell<dyn Shape>>>) -> Rc<RefCell<dyn Shape>> {
    if its.is_empty() {
        return Null::new()
    } 

    if let ShapeType::AllNodes = its[0].borrow().shape_type() {
        return join(its[1..].to_vec())
    }

    return Intersect::new(its)
}


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

    fn apply(&self, shape: Rc<RefCell<dyn Shape>>, ctx: PathContext) -> (Rc<RefCell<dyn Shape>>, PathContext) {
        if self.nodes.is_empty() {
            return (shape, ctx)
        }
        let s = Lookup::new(self.nodes.clone());
        if let ShapeType::AllNodes = shape.borrow().shape_type() {

        }
        return (join(vec![s, shape]), ctx)
    }
}


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

    fn apply(&self, r#in: Rc<RefCell<dyn Shape>>, ctx: PathContext) -> (Rc<RefCell<dyn Shape>>, PathContext) {
        return (new_in_out(r#in, self.via.as_shape(), Some(ctx.label_set), self.tags, true), ctx)
    }
}



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

    fn apply(&self, out: Rc<RefCell<dyn Shape>>, ctx: PathContext) -> (Rc<RefCell<dyn Shape>>, PathContext) {
        return (new_in_out(out, self.via.as_shape(), Some(ctx.label_set), self.tags, false), ctx)
    }
}



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

    fn apply(&self, shape: Rc<RefCell<dyn Shape>>, ctx: PathContext) -> (Rc<RefCell<dyn Shape>>, PathContext) {
        let via = self.via.as_shape();
        return (Rc::new(RefCell::new(Union(vec![
            new_in_out(shape, self.via.as_shape(), Some(ctx.label_set), self.tags, true),
            new_in_out(shape, self.via.as_shape(), Some(ctx.label_set), self.tags, false)
        ]))), ctx)
    }
}





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
        (FollowMorphism::new(self.path.reverse()), None)
    }

    fn apply(&self, shape: Rc<RefCell<dyn Shape>>, ctx: PathContext) -> (Rc<RefCell<dyn Shape>>, PathContext) {
        (self.path.shape_from(shape), ctx)
    }
}
