use super::{Shape, Base, Index, Scanner, Costs, Null, ShapeType};
use super::super::refs;
use super::super::value::Value;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use io_context::Context;
use std::fmt;

pub struct Resolver {
    qs: Rc<dyn refs::Namer>,
    order: Vec<Value>
}

impl Resolver {
    pub fn new(qs: Rc<dyn refs::Namer>, nodes: Vec<Value>) -> Rc<RefCell<Resolver>> {
        Rc::new(RefCell::new(Resolver {
            qs: qs,
            order: nodes,
        }))
    }
}


impl fmt::Display for Resolver {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Resolver")
    }
}


impl Shape for Resolver {
    fn iterate(&self) -> Rc<RefCell<dyn Scanner>> {
        ResolverNext::new(self.qs.clone(), &self.order)
    }

    fn lookup(&self) -> Rc<RefCell<dyn Index>> {
        ResolverContains::new(self.qs.clone(), &self.order)
    }

    #[allow(unused)]
    fn stats(&mut self, ctx: &Context) -> Result<Costs, String> {
        return Ok(Costs {
            next_cost: 1,
            contains_cost: 1,
            size: refs::Size {
                value: self.order.len() as i64,
                exact: true
            } 
        })
    }

    #[allow(unused)]
    fn optimize(&mut self, ctx: &Context) -> Option<Rc<RefCell<dyn Shape>>> {
        if self.order.is_empty() {
            return Some(Null::new())
        }
        return None
    }

    fn sub_iterators(&self) -> Option<Vec<Rc<RefCell<dyn Shape>>>> {
        None
    }

    fn shape_type(&mut self) -> ShapeType {
        ShapeType::Resolver
    }
}



struct ResolverNext {
    qs: Rc<dyn refs::Namer>,
    order: Vec<Value>,
    values: Vec<refs::Ref>,
    cached: bool,
    index: usize,
    err: Option<String>,
    result: Option<refs::Ref>
}

impl ResolverNext {
    fn new(qs: Rc<dyn refs::Namer>, nodes: &Vec<Value>) -> Rc<RefCell<ResolverNext>> {
        Rc::new(RefCell::new(ResolverNext {
            qs: qs.clone(),
            order: nodes.clone(),
            values: Vec::new(),
            cached: false,
            index: 0,
            err: None,
            result: None
        }))
    }

    fn resolve(&mut self, ctx: &Context) -> Result<(), String> {
        let values = self.qs.refs_of(ctx, &self.order)?;
        self.values = Vec::new();

        for value in values {
            self.values.push(value.clone());
        }

        self.order = Vec::new();
        self.cached = true;
        return Ok(())
    }
}

impl fmt::Display for ResolverNext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ResolverNext")
    }
}

impl Base for ResolverNext {

    #[allow(unused)]
    fn tag_results(&self, tags: &mut HashMap<String, refs::Ref>) {}

    fn result(&self) -> Option<refs::Ref> {
        return self.result.clone()
    }

    #[allow(unused)]
    fn next_path(&mut self, ctx: &Context) -> bool {
        return false
    }

    fn err(&self) -> Option<String> {
        return self.err.clone()
    }

    fn close(&mut self) -> Result<(), String> {
        return Ok(())
    }
}

impl Scanner for ResolverNext {
    fn next(&mut self, ctx: &Context) -> bool {
        if !self.cached {
            self.err = if let Err(e) = self.resolve(ctx) { Some(e) } else { None };
            if self.err.is_some() {
                return false
            }
        }
        if self.index >= self.values.len() {
            self.result = None;
            return false
        }
        self.result = Some(self.values[self.index].clone());
        self.index += 1;
        return true
    }

}



struct ResolverContains {
    qs: Rc<dyn refs::Namer>,
    order: Vec<Value>,
    nodes: HashMap<Value, Value>,
    cached: bool,
    err: Option<String>,
    result: Option<refs::Ref>
}

impl ResolverContains {
    fn new(qs: Rc<dyn refs::Namer>, nodes: &Vec<Value>) -> Rc<RefCell<ResolverContains>> {
        Rc::new(RefCell::new(ResolverContains {
           qs: qs,
           order: nodes.clone(),
           nodes: HashMap::new(),
           cached: false,
           err: None,
           result: None
       }))
    }

    fn resolve(&mut self, ctx: &Context) -> Result<(), String> {
        let values = self.qs.refs_of(ctx, &self.order)?;

        self.nodes = HashMap::new();

        for (index, value) in values.iter().enumerate() {
            let node = self.order.get(index).unwrap();
            self.nodes.insert(value.key.clone(), node.clone());
        }

        self.order = Vec::new();
        self.cached = true;
        return Ok(())
    }
}


impl fmt::Display for ResolverContains {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ResolverContains")
    }
}


impl Base for ResolverContains {
    #[allow(unused)]
    fn tag_results(&self, tags: &mut HashMap<String, refs::Ref>) {}

    fn result(&self) -> Option<refs::Ref> {
        return self.result.clone()
    }

    #[allow(unused)]
    fn next_path(&mut self, ctx: &Context) -> bool {
        return false
    }

    fn err(&self) -> Option<String> {
        return self.err.clone()
    }

    fn close(&mut self) -> Result<(), String> {
        return Ok(())
    }
}

impl Index for ResolverContains {
    fn contains(&mut self, ctx: &Context, v:&refs::Ref) -> bool {
        if !self.cached {
            self.err = if let Err(e) = self.resolve(ctx) { Some(e) } else { None };
            if self.err.is_some() {
                return false
            }
        }

        let has = self.nodes.contains_key(&v.key);

        if has {
            self.result = Some(v.clone());
        }

        return has;
    }
}