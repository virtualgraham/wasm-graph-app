
use super::path;
use std::rc::Rc;
use std::cell::RefCell;
use crate::graph::quad::{QuadStore, QuadWriter, IgnoreOptions, Quad};
use crate::graph::refs::Ref;
use crate::graph::graphmock;
use crate::graph::value::{Value, Values};
use crate::graph::iterator;
use io_context::Context;
use std::collections::HashMap;

pub fn new_memory_graph() -> Foo {
    let qs = Rc::new(RefCell::new(graphmock::Store::new()));

    let s = Rc::new(RefCell::new(Session {
        ctx: Rc::new(RefCell::new(Context::background())),
        qs: qs.clone(),
        qw: QuadWriter::new(qs.clone(), IgnoreOptions{ignore_dup: true, ignore_missing: true}),
        limit: -1
    }));

    let g = Graph::new(s.clone());

    Foo {
        g,
        s
    }
}


pub struct Foo {
    pub g: Graph,
    pub s: Rc<RefCell<Session>>
}

impl Foo {
    pub fn graph(self) -> Graph {
        return self.g;
    }

    pub fn g(self) -> Graph {
        return self.g;
    }
}

pub struct Session {
    ctx: Rc<RefCell<Context>>,
    qs: Rc<RefCell<dyn QuadStore>>,
    qw: QuadWriter,
    limit: i64
}

// struct RunIteratorTagMapCallback<'a>(&'a mut Session);

// impl<'a> iterator::iterate::TagMapCallback for RunIteratorTagMapCallback<'a> {
//     fn tag_map_callback(&mut self, tags: HashMap<String, Ref>) -> bool {
//         let ctx = self.0.ctx.clone();

//         if !self.0.send(QueryResult{meta: false, val: None, tags: tags}) {
//             self.0.ctx.borrow_mut().add_cancel_signal().cancel();
//             return true;
//         }

//         return false
//     }
// }

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



    fn run_iterator(&mut self, it: Rc<RefCell<dyn iterator::Shape>>) -> iterator::iterate::Chain {
        iterator::iterate::Chain::new(self.ctx.clone(), it, false, self.limit, true)
    }

}




struct QueryResult {
    meta: bool,
    val: Option<Value>,
    tags: HashMap<String, Ref>
}

pub struct Graph {
    session: Rc<RefCell<Session>>,
    path: Option<Path>
}


impl Graph {
    pub fn new(session: Rc<RefCell<Session>>) -> Graph {
        Graph {
            path: None,
            session
        }
    }

    pub fn v<V: Into<Values>>(mut self, qv: V) -> Path {
        self.path = Some(
            Path::new(
                self.session.clone(), 
                true, 
                path::Path::start_path(
                    Some(
                        self.session.borrow().qs.clone()
                    ), 
                    qv.into().to_vec()
                )
            )
        );
        return self.path.unwrap();
    }

    pub fn m(mut self) -> Path {
        self.path = Some(Path::new(self.session.clone(), false, path::Path::start_morphism(Vec::new())));
        return self.path.unwrap();
    }
}


#[derive(Clone)]
pub struct Path {
    pub session: Rc<RefCell<Session>>,
    finals: bool,
    path: path::Path
}

impl Path {
    fn new(session: Rc<RefCell<Session>>, finals: bool, path: path::Path) -> Path {
        Path {
            session,
            finals,
            path
        }
    }

    fn clonePath(mut self) -> path::Path {
        self.path = self.path.clone();
        return self.path
    }

    fn build_iterator_tree(&self) -> Rc<RefCell<dyn iterator::Shape>> {
        let s = self.session.borrow();
        let ctx = s.ctx.borrow();

        let qs = self.session.borrow().qs.clone();

        self.path.build_iterator_on(&*ctx, qs)
    }

    ///////////////
    // Finals
    ///////////////


    pub fn get_limit(&self, limit: i64) -> iterator::iterate::Chain {
        let it = self.build_iterator_tree();
        let it = iterator::save::tag(&it, &"id");

        self.session.borrow_mut().limit = limit;
        self.session.borrow_mut().run_iterator(it)
    }

    pub fn all(&mut self) -> iterator::iterate::Chain {
        let limit = self.session.borrow().limit;
        self.get_limit(limit)
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
        Path::new(self.session, self.finals, np)
    }


    // fn _in_out_values(self, values: Vec<Value>, tags: Option<Vec<String>>, dir_in: bool) -> Path {
    //     let tags:Vec<String> = if let Some(t) = tags { t } else { Vec::new() };
    //     let via = path::Via::Values(values);

    //     let np = if dir_in { self.path.in_with_tags(tags, via) } else { self.path.out_with_tags(tags, via) };
        
    //     Path::new(self.session, self.finals, np)
    // }

    // fn _in_out_path(self, path: &Path, tags: Option<Vec<String>>, dir_in: bool) -> Path {
    //     let tags:Vec<String> = if let Some(t) = tags { t } else { Vec::new() };
    //     let via = path::Via::Path(path.path.clone());

    //     let np = if dir_in { self.path.in_with_tags(tags, via) } else { self.path.out_with_tags(tags, via) };
        
    //     Path::new(self.session, self.finals, np)
    // }


    ///////////////////////////
    // In(values: String[], tags: String[])
    ///////////////////////////
    pub fn r#in<V: Into<path::Via>>(self, via: V, tags: Option<Vec<String>>) -> Path {
        let tags:Vec<String> = if let Some(t) = tags { t } else { Vec::new() };
        let np = self.path.in_with_tags(tags, via.into());
        Path::new(self.session, self.finals, np)
    }


    // ///////////////////////////
    // // In(path: Path, tags: String[])
    // ///////////////////////////
    // pub fn in_path(self, path: &Path, tags: Option<Vec<String>>) -> Path {
    //     self._in_out_path(path, tags, true)
    // }


    ///////////////////////////
    // Out(values: String[], tags: String[])
    ///////////////////////////
    pub fn out<V: Into<path::Via>>(self, via: V, tags: Option<Vec<String>>) -> Path {
        let tags:Vec<String> = if let Some(t) = tags { t } else { Vec::new() };
        let np = self.path.out_with_tags(tags, via.into());
        Path::new(self.session, self.finals, np)
    }


    // ///////////////////////////
    // // Out(path: Path, tags: String[])
    // ///////////////////////////
    // pub fn out_path(self, path: &Path, tags: Option<Vec<String>>) -> Path {
    //     self._in_out_path(path, tags, false)
    // }


    ///////////////////////////
    // Both(values: String[], tags: String[])
    ///////////////////////////
    pub fn both_values(self, values: Vec<Value>, tags: Option<Vec<String>>) -> Path {
        let tags:Vec<String> = if let Some(t) = tags { t } else { Vec::new() };
        let via = path::Via::Values(values);
        
        Path::new(self.session, self.finals, self.path.both_with_tags(tags, via))
    }


    ///////////////////////////
    // Both(path: Path, tags: String[])
    ///////////////////////////
    pub fn both_path(self, path: &Path, tags: Option<Vec<String>>) -> Path {
        let tags:Vec<String> = if let Some(t) = tags { t } else { Vec::new() };
        let via = path::Via::Path(path.path.clone());
        
        Path::new(self.session, self.finals, self.path.both_with_tags(tags, via))
    }


    ///////////////////////////
    // Follow(path: Path)
    ///////////////////////////
    pub fn follow(self, ep: &Path) -> Path {
        return Path::new(self.session, self.finals, self.path.follow(ep.path.clone()))
    }


    ///////////////////////////
    // FollowR(path: Path)
    ///////////////////////////
    pub fn follow_r(self, ep: &Path) -> Path {
        return Path::new(self.session, self.finals, self.path.follow_reverse(ep.path.clone()))
    }


    ///////////////////////////
    // FollowRecursive(path: Path, maxDepth: int, tags: Stringp[])
    ///////////////////////////
    pub fn follow_recursive_path(self, path: &Path, max_depth: Option<i32>, tags: Option<Vec<String>>) -> Path {
        let tags:Vec<String> = if let Some(t) = tags { t } else { Vec::new() };
        let via = path::Via::Path(path.path.clone());
        let max_depth = match max_depth { Some(d) => d, None => 50 };
        return Path::new(self.session, self.finals, self.path.follow_recursive(via, max_depth, tags))
    }


    ///////////////////////////
    // FollowRecursive(value: String, maxDepth: int, tags: Stringp[])
    ///////////////////////////
    pub fn follow_recursive_value(self, value: Value, max_depth: Option<i32>, tags: Option<Vec<String>>) -> Path {
        let tags:Vec<String> = if let Some(t) = tags { t } else { Vec::new() };
        let via = path::Via::Values(vec![value]);
        let max_depth = match max_depth { Some(d) => d, None => 50 };
        return Path::new(self.session, self.finals, self.path.follow_recursive(via, max_depth, tags))
    }


    ///////////////////////////
    // And(path: Path)
    // Intersect(path: Path)
    ///////////////////////////
    pub fn intersect(self, path: &Path) -> Path {
        return Path::new(self.session, self.finals, self.path.and(path.path.clone()))
    }


    ///////////////////////////
    // Or(path: Path)
    // Union(path: Path)
    ///////////////////////////
    pub fn union(self, path: &Path) -> Path {
        return Path::new(self.session, self.finals, self.path.or(path.path.clone()))
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



