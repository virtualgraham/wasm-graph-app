
use super::path;
use std::rc::Rc;
use std::cell::RefCell;
use crate::graph::quad::{QuadStore, QuadWriter, IgnoreOptions, Quad};
use crate::graph::refs::Ref;
use crate::graph::graphmock;
use crate::graph::value::Value;
use crate::graph::iterator;
use io_context::Context;
use std::collections::HashMap;


pub fn new_memory_graph() -> Session {
    let qs = Rc::new(RefCell::new(graphmock::Store::new()));

    Session {
        g: Graph::new(qs.clone()),
        ctx: Rc::new(RefCell::new(Context::background())),
        qs: qs.clone(),
        qw: QuadWriter::new(qs.clone(), IgnoreOptions{ignore_dup: true, ignore_missing: true}),
        limit: 1000,
        count: 0,
        out: None
    }
}



pub struct Session {
    g: Graph,
    ctx: Rc<RefCell<Context>>,
    qs: Rc<RefCell<dyn QuadStore>>,
    qw: QuadWriter,
    limit: u64,
    count: u64,
    out: Option<SessionResult>
}



impl Session {
    pub fn write(&self, quads: Vec<Quad>) {
        for quad in &quads {
            self.qw.add_quad(quad.clone()).unwrap();
        }
    }

    pub fn read(&self) -> Vec<Quad> {
        vec![Quad::new("a", "b", "c", "d")]
    }

    pub fn delete(&self, quads: Vec<Quad>) {

    }

    pub fn graph(self) -> Graph {
        return self.g;
    }

    pub fn g(self) -> Graph {
        return self.g;
    }

    fn run_iterator(&mut self, it: Rc<RefCell<dyn iterator::Shape>>) -> Result<(), String> {
        let mut stop = false;
        let res = iterator::iterate::Chain::new(self.ctx.clone(), it).paths(true).tag_each(self);
        if stop {
            return Ok(())
        }
        return res
    }

    // TODO: This should be a lambda so we need to fix this because it violates encapsulation
    pub fn do_tag(&mut self, tags: HashMap<String, Ref>) -> bool {
        let ctx = self.ctx.clone();

        if !self.send(SessionResult{meta: false, val: None, tags: tags}) {
            self.ctx.borrow_mut().add_cancel_signal().cancel();
            return true;
        }

        return false
    }

    fn send(&mut self, result: SessionResult) -> bool {
        if self.limit > 0 && self.count >= self.limit {
            return false
        }
        if self.out.is_none() {
            return false
        }
        
        self.out = Some(result);

        if let Some(reason) = self.ctx.borrow().done() {
            return false
        }
        self.count += 1;
        return self.limit <= 0 || self.count < self.limit
    }
}

struct SessionResult {
    meta: bool,
    val: Option<Value>,
    tags: HashMap<String, Ref>
}

pub struct Graph {
    path: Option<Path>,
    qs: Rc<RefCell<dyn QuadStore>>
}


impl Graph {
    pub fn new(qs: Rc<RefCell<dyn QuadStore>>) -> Graph {
        Graph {
            path: None,
            qs
        }
    }

    pub fn v(mut self, qv: Option<Vec<Value>>) -> Path {
        let qv = match qv { Some(v) => v, None => Vec::new() };
        self.path = Some(Path::new(true, path::Path::start_morphism(qv)));
        return self.path.unwrap();
    }

    pub fn m(mut self) -> Path {
        self.path = Some(Path::new(false, path::Path::start_morphism(Vec::new())));
        return self.path.unwrap();
    }
}


#[derive(Clone)]
pub struct Path {
    finals: bool,
    path: path::Path
}

impl Path {
    fn new(finals: bool, path: path::Path) -> Path {
        Path {
            finals,
            path
        }
    }

    fn clonePath(mut self) -> path::Path {
        self.path = self.path.clone();
        return self.path
    }

    fn build_iterator_tree(&self) -> Rc<RefCell<dyn iterator::Shape>> {
        let s = &*self.path.session.as_ref().unwrap().borrow();
        
        let ctx = s.ctx.borrow();
        let qs = s.qs.clone();

        self.path.build_iterator_on(&*ctx, qs)
    }

    ///////////////
    // Finals
    ///////////////


    pub fn get_limit(&self, limit: u64) {
        let it = self.build_iterator_tree();
        let it = iterator::save::tag(&it, &"id");

        let s = &mut*self.path.session.as_ref().unwrap().borrow_mut();
        s.limit = limit;
        s.count = 0;
        s.run_iterator(it);
    }

    pub fn all(self) {
        self.get_limit(self.path.session.as_ref().unwrap().borrow().limit)
    }

    // pub fn to_array(self, args: Option<Box<[JsValue]>>) -> JsValue {
    //     JsValue::NULL
    // }

    // //#[wasm_bindgen(js_name = tagArray)]
    // pub fn tag_array(self, args: Option<Box<[JsValue]>>) -> JsValue {
    //     JsValue::NULL
    // }

    // //#[wasm_bindgen(js_name = toValue)]
    // pub fn to_value(self) -> JsValue {
    //     JsValue::NULL
    // }

    // //#[wasm_bindgen(js_name = tagValue)]
    // pub fn tag_value(self) -> JsValue {
    //     JsValue::NULL
    // }

    // pub fn map(self, callback: &JsValue) {

    // }

    // //#[wasm_bindgen(js_name = forEach)]
    // pub fn for_each(self, callback: &JsValue) {

    // }

    // pub fn count(self) -> JsValue {
    //     JsValue::NULL
    // }


    ///////////////
    // Traversals
    ///////////////
    

    ///////////////////////////
    // Is(nodes: String[])
    ///////////////////////////
    pub fn is(self, nodes: Vec<Value>) -> Path {
        let np = self.path.is(nodes);
        Path::new(self.finals, np)
    }


    fn _in_out_values(self, values: Vec<Value>, tags: Option<Vec<String>>, dir_in: bool) -> Path {
        let tags:Vec<String> = if let Some(t) = tags { t } else { Vec::new() };
        let via = path::Via::Values(values);

        let np = if dir_in { self.path.in_with_tags(tags, via) } else { self.path.out_with_tags(tags, via) };
        
        Path::new(self.finals, np)
    }

    fn _in_out_path(self, path: &Path, tags: Option<Vec<String>>, dir_in: bool) -> Path {
        let tags:Vec<String> = if let Some(t) = tags { t } else { Vec::new() };
        let via = path::Via::Path(path.path.clone());

        let np = if dir_in { self.path.in_with_tags(tags, via) } else { self.path.out_with_tags(tags, via) };
        
        Path::new(self.finals, np)
    }


    ///////////////////////////
    // In(values: String[], tags: String[])
    ///////////////////////////
    pub fn in_values(self, values: Vec<Value>, tags: Option<Vec<String>>) -> Path {
        self._in_out_values(values, tags, true)
    }


    ///////////////////////////
    // In(path: Path, tags: String[])
    ///////////////////////////
    pub fn in_path(self, path: &Path, tags: Option<Vec<String>>) -> Path {
        self._in_out_path(path, tags, true)
    }


    ///////////////////////////
    // Out(values: String[], tags: String[])
    ///////////////////////////
    pub fn out_values(self, values: Vec<Value>, tags: Option<Vec<String>>) -> Path {
        self._in_out_values(values, tags, false)
    }


    ///////////////////////////
    // Out(path: Path, tags: String[])
    ///////////////////////////
    pub fn out_path(self, path: &Path, tags: Option<Vec<String>>) -> Path {
        self._in_out_path(path, tags, false)
    }


    ///////////////////////////
    // Both(values: String[], tags: String[])
    ///////////////////////////
    pub fn both_values(self, values: Vec<Value>, tags: Option<Vec<String>>) -> Path {
        let tags:Vec<String> = if let Some(t) = tags { t } else { Vec::new() };
        let via = path::Via::Values(values);
        
        Path::new(self.finals, self.path.both_with_tags(tags, via))
    }


    ///////////////////////////
    // Both(path: Path, tags: String[])
    ///////////////////////////
    pub fn both_path(self, path: &Path, tags: Option<Vec<String>>) -> Path {
        let tags:Vec<String> = if let Some(t) = tags { t } else { Vec::new() };
        let via = path::Via::Path(path.path.clone());
        
        Path::new(self.finals, self.path.both_with_tags(tags, via))
    }


    ///////////////////////////
    // Follow(path: Path)
    ///////////////////////////
    pub fn follow(self, ep: &Path) -> Path {
        return Path::new(self.finals, self.path.follow(ep.path.clone()))
    }


    ///////////////////////////
    // FollowR(path: Path)
    ///////////////////////////
    pub fn follow_r(self, ep: &Path) -> Path {
        return Path::new(self.finals, self.path.follow_reverse(ep.path.clone()))
    }


    ///////////////////////////
    // FollowRecursive(path: Path, maxDepth: int, tags: Stringp[])
    ///////////////////////////
    pub fn follow_recursive_path(self, path: &Path, max_depth: Option<i32>, tags: Option<Vec<String>>) -> Path {
        let tags:Vec<String> = if let Some(t) = tags { t } else { Vec::new() };
        let via = path::Via::Path(path.path.clone());
        let max_depth = match max_depth { Some(d) => d, None => 50 };
        return Path::new(self.finals, self.path.follow_recursive(via, max_depth, tags))
    }


    ///////////////////////////
    // FollowRecursive(value: String, maxDepth: int, tags: Stringp[])
    ///////////////////////////
    pub fn follow_recursive_value(self, value: Value, max_depth: Option<i32>, tags: Option<Vec<String>>) -> Path {
        let tags:Vec<String> = if let Some(t) = tags { t } else { Vec::new() };
        let via = path::Via::Values(vec![value]);
        let max_depth = match max_depth { Some(d) => d, None => 50 };
        return Path::new(self.finals, self.path.follow_recursive(via, max_depth, tags))
    }


    ///////////////////////////
    // And(path: Path)
    // Intersect(path: Path)
    ///////////////////////////
    pub fn intersect(self, path: &Path) -> Path {
        return Path::new(self.finals, self.path.and(path.path.clone()))
    }


    ///////////////////////////
    // Or(path: Path)
    // Union(path: Path)
    ///////////////////////////
    pub fn union(self, path: &Path) -> Path {
        return Path::new(self.finals, self.path.or(path.path.clone()))
    }


    ///////////////////////////
    // Back(tag: String)
    ///////////////////////////
    pub fn back(self, tag: String) -> Path {
        self
    }

    ///////////////////////////
    // Back(tags: String[])
    ///////////////////////////
    pub fn tag(self, tags: Vec<String>) -> Path {
        self
    }

    ///////////////////////////
    // As(tags: String[])
    ///////////////////////////
    pub fn r#as(self, tags: Vec<String>) -> Path {
        self
    }

    ///////////////////////////
    // Has(predicate: String, object: String)
    ///////////////////////////

    ///////////////////////////
    // *Has(predicate: Path, object: String)
    // *Has(predicate: String, filters: Filter[])
    // *Has(predicate: Path, filters: Filter[])
    ///////////////////////////
    pub fn has(self, predicate: String, object: String) -> Path {
        self
    }

    ///////////////////////////
    // HasR(predicate: String, object: String)
    ///////////////////////////
    
    ///////////////////////////
    // *HasR(predicate: Path, object: String)
    // *HasR(predicate: String, filters: Filter[])
    // *HasR(predicate: Path, filters: Filter[])
    ///////////////////////////
    pub fn has_r(self, predicate: String, object: String) -> Path {
        self
    }


    ///////////////////////////
    // Save(predicate: String, tag: String)
    ///////////////////////////
    pub fn save(self, predicate: String, tag: String) -> Path {
        self
    }

    ///////////////////////////
    // SaveR(predicate: String, tag: String)
    ///////////////////////////
    pub fn save_r(self, predicate: String, tag: String) -> Path {
        self
    }

    ///////////////////////////
    // SaveOpt(predicate: String, tag: String)
    ///////////////////////////
    pub fn save_opt(self, predicate: String, tag: String) -> Path {
        self
    }

    ///////////////////////////
    // SaveOptR(predicate: String, tag: String)
    ///////////////////////////
    pub fn save_opt_r(self, predicate: String, tag: String) -> Path {
        self
    }

    ///////////////////////////
    // Except(path: Path)
    ///////////////////////////
    pub fn except(self, path: &Path) -> Path {
        self
    }

    ///////////////////////////
    // Unique()
    ///////////////////////////
    pub fn unique(self) -> Path {
        self
    }

    ///////////////////////////
    // Difference(path: Path)
    ///////////////////////////
    pub fn difference(self, path: &Path) -> Path {
        self
    }

    ///////////////////////////
    // Labels()
    ///////////////////////////
    pub fn labels(self) -> Path {
        self
    }

    ///////////////////////////
    // InPredicates(tag:String)
    ///////////////////////////
    pub fn in_predicates(self, tag: String) -> Path {
        self
    }

    ///////////////////////////
    // OutPredicates()
    ///////////////////////////
    pub fn out_predicates(self) -> Path {
        self
    }

    ///////////////////////////
    // SaveInPredicates(tag:String)
    ///////////////////////////
    pub fn save_in_predicates(self, tag: String) -> Path {
        self
    }

    ///////////////////////////
    // SaveOutPredicates(tag:String)
    ///////////////////////////
    pub fn save_out_predicates(self, tag: String) -> Path {
        self
    }


    ///////////////////////////
    // LabelContext(values: String[], tags: String[])
    ///////////////////////////
    pub fn label_context_values(self, values: Vec<String>, tags: Vec<String>) -> Path {
        self
    }

    ///////////////////////////
    // LabelContext(path: Path, tags: String[])
    ///////////////////////////
    pub fn label_context_path(self, path: &Path, tags: Vec<String>) -> Path {
        self
    }


    ///////////////////////////
    // Filter(filter: Filter)
    ///////////////////////////
    pub fn filter(self) -> Path {
        self
    }

    ///////////////////////////
    // Limit(limit: Number)
    ///////////////////////////
    pub fn limit(self, limit: i32) -> Path {
        self
    }

    ///////////////////////////
    // Skip(offset: Number)
    ///////////////////////////
    pub fn skip(self, offset: i32) -> Path {
        self
    }

    ///////////////////////////
    // Order()
    ///////////////////////////
    pub fn order(self) -> Path {
        self
    }


}



