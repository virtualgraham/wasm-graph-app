
use crate::graph::value::Value;
use std::cell::RefCell;
use std::rc::Rc;
use super::path::PathContext;
use crate::query::shape::*;
use crate::query::path::{Via, Path};


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

pub trait Morphism {
    fn reversal(&self, ctx: &mut PathContext) -> (Rc<dyn Morphism>, Option<PathContext>);
    fn apply(&self, shape: Rc<RefCell<dyn Shape>>, ctx: &mut PathContext) -> (Rc<RefCell<dyn Shape>>, Option<PathContext>);
    fn is_tag(&self) -> bool { false }
    fn tags(&self) -> Option<Vec<String>> { None }
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
    fn reversal(&self, ctx: &mut PathContext) -> (Rc<dyn Morphism>, Option<PathContext>) {
        (IsMorphism::new(self.nodes.clone()), None)
    }

    fn apply(&self, shape: Rc<RefCell<dyn Shape>>, ctx: &mut PathContext) -> (Rc<RefCell<dyn Shape>>, Option<PathContext>) {
        println!("IsMorphism apply() {:?}", self.nodes);
        if self.nodes.is_empty() {
            return (shape, None)
        }
        let s = Lookup::new(self.nodes.clone());
        if let ShapeType::AllNodes = shape.borrow_mut().shape_type() {
            println!("IsMorphism AllNodes Shape type");
            return (s, None)
        }
        return (join(vec![s, shape]), None)
    }
}

//////////////////////////////////////////////////////////

pub struct InMorphism {
    tags: Option<Vec<String>>,
    via: Via
}

impl InMorphism {
    pub fn new(tags: Option<Vec<String>>, via: Via) -> Rc<dyn Morphism> {
        Rc::new(InMorphism {
            tags, 
            via
        })
    }
}

impl Morphism for InMorphism {
    fn reversal(&self, ctx: &mut PathContext) -> (Rc<dyn Morphism>, Option<PathContext>) {
        (OutMorphism::new(self.tags.clone(), self.via.clone()), None)
    }

    fn apply(&self, shape: Rc<RefCell<dyn Shape>>, ctx: &mut PathContext) -> (Rc<RefCell<dyn Shape>>, Option<PathContext>) {
        println!("InMorphism apply()");
        return (new_in_out(shape, self.via.as_shape(), ctx.label_set.clone(), self.tags.clone(), true), None)
    }

    fn tags(&self) -> Option<Vec<String>> { 
        self.tags.clone()
    }
}

//////////////////////////////////////////////////////////

pub struct OutMorphism {
    tags: Option<Vec<String>>,
    via: Via
}

impl OutMorphism {
    pub fn new(tags: Option<Vec<String>>, via: Via) -> Rc<dyn Morphism> {
        Rc::new(OutMorphism {
            tags, 
            via
        })
    }
}

impl Morphism for OutMorphism {
    fn reversal(&self, ctx: &mut PathContext) -> (Rc<dyn Morphism>, Option<PathContext>) {
        (InMorphism::new(self.tags.clone(), self.via.clone()), None)
    }

    fn apply(&self, shape: Rc<RefCell<dyn Shape>>, ctx: &mut PathContext) -> (Rc<RefCell<dyn Shape>>, Option<PathContext>) {
        println!("OutMorphism apply()");
        return (new_in_out(shape, self.via.as_shape(), ctx.label_set.clone(), self.tags.clone(), false), None)
    }

    fn tags(&self) -> Option<Vec<String>> { 
        self.tags.clone()
    }
}

//////////////////////////////////////////////////////////

pub struct BothMorphism {
    tags: Option<Vec<String>>,
    via: Via
}

impl BothMorphism {
    pub fn new(tags: Option<Vec<String>>, via: Via) -> Rc<dyn Morphism> {
        Rc::new(BothMorphism {
            tags, 
            via
        })
    }
}

impl Morphism for BothMorphism {
    fn reversal(&self, ctx: &mut PathContext) -> (Rc<dyn Morphism>, Option<PathContext>) {
        (BothMorphism::new(self.tags.clone(), self.via.clone()), None)
    }

    fn apply(&self, shape: Rc<RefCell<dyn Shape>>, ctx: &mut PathContext) -> (Rc<RefCell<dyn Shape>>, Option<PathContext>) {
        println!("BothMorphism apply()");
        let via = self.via.as_shape();
        return (Rc::new(RefCell::new(Union(vec![
            new_in_out(shape.clone(), via.clone(), ctx.label_set.clone(), self.tags.clone(), true),
            new_in_out(shape.clone(), via.clone(), ctx.label_set.clone(), self.tags.clone(), false)
        ]))), None)
    }

    fn tags(&self) -> Option<Vec<String>> { 
        self.tags.clone()
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
    fn reversal(&self, ctx: &mut PathContext) -> (Rc<dyn Morphism>, Option<PathContext>) {
        (FollowMorphism::new(self.path.clone().reverse()), None)
    }

    fn apply(&self, shape: Rc<RefCell<dyn Shape>>, ctx: &mut PathContext) -> (Rc<RefCell<dyn Shape>>, Option<PathContext>) {
        println!("FollowMorphism apply()");
        (self.path.clone().shape_from(shape), None)
    }
}

//////////////////////////////////////////////////////////

pub struct FollowRecursiveMorphism {
    path: Path,
    max_depth: i32,
    depth_tags: Vec<String>,

}

impl FollowRecursiveMorphism {
    pub fn new(path: Path, max_depth: i32, depth_tags: Vec<String>) -> Rc<dyn Morphism> {
        Rc::new(FollowRecursiveMorphism {
            path,
            max_depth,
            depth_tags
        })
    }
}

impl Morphism for FollowRecursiveMorphism {
    fn reversal(&self, ctx: &mut PathContext) -> (Rc<dyn Morphism>, Option<PathContext>) {
        (FollowRecursiveMorphism::new(self.path.clone().reverse(), self.max_depth, self.depth_tags.clone()), None)
    }

    fn apply(&self, shape: Rc<RefCell<dyn Shape>>, ctx: &mut PathContext) -> (Rc<RefCell<dyn Shape>>, Option<PathContext>) {
        println!("FollowRecursiveMorphism apply()");
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
    fn reversal(&self, ctx: &mut PathContext) -> (Rc<dyn Morphism>, Option<PathContext>) {
        (AndMorphism::new(self.path.clone()), None)
    }

    fn apply(&self, shape: Rc<RefCell<dyn Shape>>, ctx: &mut PathContext) -> (Rc<RefCell<dyn Shape>>, Option<PathContext>) {
        println!("AndMorphism apply()");
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
    fn reversal(&self, ctx: &mut PathContext) -> (Rc<dyn Morphism>, Option<PathContext>) {
        (OrMorphism::new(self.path.clone()), None)
    }

    fn apply(&self, shape: Rc<RefCell<dyn Shape>>, ctx: &mut PathContext) -> (Rc<RefCell<dyn Shape>>, Option<PathContext>) {
        println!("OrMorphism apply()");
       (Rc::new(RefCell::new(Union(vec![shape, self.path.clone().shape()]))), None)
    }
}

//////////////////////////////////////////////////////////

pub struct FilterMorphism {
    filters: Vec<Rc<dyn ValueFilter>>
}

impl FilterMorphism {
    pub fn new(filters: Vec<Rc<dyn ValueFilter>>) -> Rc<dyn Morphism> {
        Rc::new(FilterMorphism {
            filters
        })
    }
}

impl Morphism for FilterMorphism {
    fn reversal(&self, ctx: &mut PathContext) -> (Rc<dyn Morphism>, Option<PathContext>) {
        (FilterMorphism::new(self.filters.clone()), None)
    }

    fn apply(&self, shape: Rc<RefCell<dyn Shape>>, ctx: &mut PathContext) -> (Rc<RefCell<dyn Shape>>, Option<PathContext>) {
        println!("FilterMorphism apply()");
        (Filter::new(shape, self.filters.clone()), None)
    }
}

//////////////////////////////////////////////////////////

pub struct TagMorphism {
    tags: Vec<String>
}

impl TagMorphism {
    pub fn new(tags: Vec<String>) -> Rc<dyn Morphism> {
        Rc::new(TagMorphism {
            tags
        })
    }
}

impl Morphism for TagMorphism {
    fn reversal(&self, ctx: &mut PathContext) -> (Rc<dyn Morphism>, Option<PathContext>) {
        (TagMorphism::new(self.tags.clone()), None)
    }

    fn apply(&self, shape: Rc<RefCell<dyn Shape>>, ctx: &mut PathContext) -> (Rc<RefCell<dyn Shape>>, Option<PathContext>) {
        println!("TagMorphism apply()");
        (Save::new(self.tags.clone(), Some(shape.clone())), None)
    }

    fn is_tag(&self) -> bool { 
        true
    }

    fn tags(&self) -> Option<Vec<String>> { 
        Some(self.tags.clone())
    }
}

//////////////////////////////////////////////////////////

pub struct ExceptMorphism {
    path: Path
}

impl ExceptMorphism {
    pub fn new(path: Path) -> Rc<dyn Morphism> {
        Rc::new(ExceptMorphism {
            path
        })
    }
}

impl Morphism for ExceptMorphism {
    fn reversal(&self, ctx: &mut PathContext) -> (Rc<dyn Morphism>, Option<PathContext>) {
        (ExceptMorphism::new(self.path.clone()), None)
    }

    fn apply(&self, shape: Rc<RefCell<dyn Shape>>, ctx: &mut PathContext) -> (Rc<RefCell<dyn Shape>>, Option<PathContext>) {
        println!("ExceptMorphism apply()");
        ( 
            join(
                vec![
                    shape, 
                    Rc::new(RefCell::new(Except{
                        from: Some(AllNodes::new()), 
                        exclude: Some(self.path.shape())
                    }))
                ]
            ), 
            None
        )
    }
}

//////////////////////////////////////////////////////////

pub struct UniqueMorphism ();

impl UniqueMorphism {
    pub fn new() -> Rc<dyn Morphism> {
        Rc::new(UniqueMorphism())
    }
}

impl Morphism for UniqueMorphism {
    fn reversal(&self, ctx: &mut PathContext) -> (Rc<dyn Morphism>, Option<PathContext>) {
        (UniqueMorphism::new(), None)
    }

    fn apply(&self, shape: Rc<RefCell<dyn Shape>>, ctx: &mut PathContext) -> (Rc<RefCell<dyn Shape>>, Option<PathContext>) {
        println!("UniqueMorphism apply()");
        ( 
            Rc::new(RefCell::new(Unique{
                from: shape
            })), 
            None
        )
    }
}

//////////////////////////////////////////////////////////

pub struct HasShapeMorphism {
    via: Via,
    rev: bool,
    nodes: Rc<RefCell<dyn Shape>>
}

impl HasShapeMorphism {

    pub fn new_has_morphism(via: Via, rev: bool, nodes: Vec<Value>) -> Rc<dyn Morphism> {
        let node:Rc<RefCell<dyn Shape>> = if nodes.is_empty() {
            AllNodes::new()
        } else {
            Lookup::new(nodes)
        };
        HasShapeMorphism::new(via, rev, node) 
    }

    pub fn new_has_filter_morphism(via: Via, rev: bool, nodes: Vec<Rc<dyn ValueFilter>>) -> Rc<dyn Morphism> {
        HasShapeMorphism::new(via, rev, Filter::new(AllNodes::new(), nodes)) 
    }

    pub fn new(via: Via, rev: bool, nodes: Rc<RefCell<dyn Shape>>) -> Rc<dyn Morphism> {
        Rc::new(HasShapeMorphism {
            via,
            rev,
            nodes
        })
    }
}

impl Morphism for HasShapeMorphism {
    fn reversal(&self, ctx: &mut PathContext) -> (Rc<dyn Morphism>, Option<PathContext>) {
        (HasShapeMorphism::new(self.via.clone(), self.rev, self.nodes.clone()), None)
    }

    fn apply(&self, r#in: Rc<RefCell<dyn Shape>>, ctx: &mut PathContext) -> (Rc<RefCell<dyn Shape>>, Option<PathContext>) {
        println!("HasShapeMorphism apply()");
        ( 
            has_labels(
                r#in,
                self.via.as_shape(),
                self.nodes.clone(),
                ctx.label_set.clone(),
                self.rev
            ), 
            None
        )
    }
}

//////////////////////////////////////////////////////////

pub struct LimitMorphism {
    limit: i64
}

impl LimitMorphism {
    pub fn new(limit: i64) -> Rc<dyn Morphism> {
        Rc::new(LimitMorphism {
            limit
        })
    }
}

impl Morphism for LimitMorphism {
    fn reversal(&self, ctx: &mut PathContext) -> (Rc<dyn Morphism>, Option<PathContext>) {
        (LimitMorphism::new(self.limit), None)
    }

    fn apply(&self, shape: Rc<RefCell<dyn Shape>>, ctx: &mut PathContext) -> (Rc<RefCell<dyn Shape>>, Option<PathContext>) {
        println!("LimitMorphism apply()");

        if self.limit <= 0 {
            return (shape, None)
        }

        ( 
            Rc::new(RefCell::new(Page{from: shape, limit: self.limit, skip: 0})), 
            None
        )
    }
}

//////////////////////////////////////////////////////////

pub struct SkipMorphism {
    offset: i64
}

impl SkipMorphism {
    pub fn new(offset: i64) -> Rc<dyn Morphism> {
        Rc::new(SkipMorphism {
            offset
        })
    }
}

impl Morphism for SkipMorphism {
    fn reversal(&self, ctx: &mut PathContext) -> (Rc<dyn Morphism>, Option<PathContext>) {
        (SkipMorphism::new(self.offset), None)
    }

    fn apply(&self, shape: Rc<RefCell<dyn Shape>>, ctx: &mut PathContext) -> (Rc<RefCell<dyn Shape>>, Option<PathContext>) {
        println!("SkipMorphism apply()");

        if self.offset == 0 {
            return (shape, None)
        }

        ( 
            Rc::new(RefCell::new(Page{from: shape, limit: 0, skip: self.offset})), 
            None
        )
    }
}

//////////////////////////////////////////////////////////

pub struct OrderMorphism ();

impl OrderMorphism {
    pub fn new() -> Rc<dyn Morphism> {
        Rc::new(OrderMorphism())
    }
}

impl Morphism for OrderMorphism {
    fn reversal(&self, ctx: &mut PathContext) -> (Rc<dyn Morphism>, Option<PathContext>) {
        (OrderMorphism::new(), None)
    }

    fn apply(&self, shape: Rc<RefCell<dyn Shape>>, ctx: &mut PathContext) -> (Rc<RefCell<dyn Shape>>, Option<PathContext>) {
        println!("OrderMorphism apply()");
        ( 
            Rc::new(RefCell::new(Sort{from: shape})), 
            None
        )
    }
}

//////////////////////////////////////////////////////////

pub struct SaveMorphism {
    via: Via,
    tag: String,
    rev: bool,
    opt: bool
}

impl SaveMorphism {
    pub fn new(via: Via, tag: String, rev: bool, opt: bool) -> Rc<dyn Morphism> {
        Rc::new(SaveMorphism {
            via,
            tag,
            rev,
            opt
        })
    }
}

impl Morphism for SaveMorphism {
    fn reversal(&self, ctx: &mut PathContext) -> (Rc<dyn Morphism>, Option<PathContext>) {
        (SaveMorphism::new(self.via.clone(), self.tag.clone(), self.rev, self.opt), None)
    }

    fn apply(&self, shape: Rc<RefCell<dyn Shape>>, ctx: &mut PathContext) -> (Rc<RefCell<dyn Shape>>, Option<PathContext>) {
        println!("SaveMorphism apply()");
        ( 
            save_via_labels(shape, self.via.as_shape(), ctx.label_set.clone(), self.tag.clone(), self.rev, self.opt), 
            None
        )
    }
}

//////////////////////////////////////////////////////////

pub struct PredicatesMorphism {
    rev: bool
}

impl PredicatesMorphism {
    pub fn new(rev: bool) -> Rc<dyn Morphism> {
        Rc::new(PredicatesMorphism {
            rev
        })
    }
}

impl Morphism for PredicatesMorphism {
    fn reversal(&self, ctx: &mut PathContext) -> (Rc<dyn Morphism>, Option<PathContext>) {
        (PredicatesMorphism::new(self.rev), None)
    }

    fn apply(&self, shape: Rc<RefCell<dyn Shape>>, ctx: &mut PathContext) -> (Rc<RefCell<dyn Shape>>, Option<PathContext>) {
        println!("PredicatesMorphism apply()");
        ( 
            predicates(shape, self.rev),
            None
        )
    }
}

//////////////////////////////////////////////////////////

pub struct SavePredicatesMorphism {
    tag: String,
    rev: bool
}

impl SavePredicatesMorphism {
    pub fn new(tag: String, rev: bool) -> Rc<dyn Morphism> {
        Rc::new(SavePredicatesMorphism {
            tag,
            rev
        })
    }
}

impl Morphism for SavePredicatesMorphism {
    fn reversal(&self, ctx: &mut PathContext) -> (Rc<dyn Morphism>, Option<PathContext>) {
        (SavePredicatesMorphism::new(self.tag.clone(), self.rev), None)
    }

    fn apply(&self, shape: Rc<RefCell<dyn Shape>>, ctx: &mut PathContext) -> (Rc<RefCell<dyn Shape>>, Option<PathContext>) {
        println!("SavePredicatesMorphism apply()");
        ( 
            save_predicates(shape, self.rev, self.tag.clone()),      
            None
        )
    }
}

//////////////////////////////////////////////////////////

pub struct LabelsMorphism();

impl LabelsMorphism {
    pub fn new() -> Rc<dyn Morphism> {
        Rc::new(LabelsMorphism())
    }
}

impl Morphism for LabelsMorphism {
    fn reversal(&self, ctx: &mut PathContext) -> (Rc<dyn Morphism>, Option<PathContext>) {
        panic!("not implemented")
    }

    fn apply(&self, shape: Rc<RefCell<dyn Shape>>, ctx: &mut PathContext) -> (Rc<RefCell<dyn Shape>>, Option<PathContext>) {
        println!("LabelsMorphism apply()");
        ( 
            labels(shape),      
            None
        )
    }
}

//////////////////////////////////////////////////////////

pub struct LabelContextMorphism {
    tags: Vec<String>,
    via: Via,
    path: Option<Rc<RefCell<dyn Shape>>>
}

impl LabelContextMorphism {
    pub fn new(via: Via, tags: Vec<String>) -> Rc<dyn Morphism> {
        let path = if let Via::None = via {
            None
        } else {
            Some(via.as_shape())
        };

        Rc::new(LabelContextMorphism {
            tags,
            via,
            path
        })
    }
}

impl Morphism for LabelContextMorphism {
    fn reversal(&self, ctx: &mut PathContext) -> (Rc<dyn Morphism>, Option<PathContext>) {
        let out = ctx.clone();
        ctx.label_set = self.path.clone();
        ( LabelContextMorphism::new(self.via.clone(), self.tags.clone()), Some(out) )
    }

    fn apply(&self, shape: Rc<RefCell<dyn Shape>>, ctx: &mut PathContext) -> (Rc<RefCell<dyn Shape>>, Option<PathContext>) {
        println!("LabelContextMorphism apply()");
        let mut out = ctx.clone();
        out.label_set = self.path.clone();
        ( shape, Some(out) )
    }
}

//////////////////////////////////////////////////////////