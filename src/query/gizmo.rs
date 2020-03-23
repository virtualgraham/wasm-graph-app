
use super::path;
use super::shape;
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
    pub fn graph(&mut self) -> &mut Graph {
        return &mut self.g;
    }

    pub fn g(&mut self) -> &mut Graph {
        return &mut self.g;
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
        iterator::iterate::Chain::new(self.ctx.clone(), it, self.qs.clone(), false, self.limit, true)
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

    pub fn v<V: Into<Values>>(&mut self, qv: V) -> &mut Path {
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
        return self.path.as_mut().unwrap();
    }

    pub fn m(&mut self) -> &mut Path {
        self.path = Some(Path::new(self.session.clone(), false, path::Path::start_morphism(Vec::new())));
        return self.path.as_mut().unwrap();
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
    pub fn is<V: Into<Values>>(&mut self, nodes: V) -> &mut Path {
        self.path.is(nodes.into().to_vec());
        self
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
    pub fn r#in<V: Into<path::Via>>(&mut self, via: V, tags: Option<Vec<String>>) -> &mut Path {
        let tags:Vec<String> = if let Some(t) = tags { t } else { Vec::new() };
        self.path.in_with_tags(tags, via.into());
        self
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
    pub fn out<V: Into<path::Via>>(&mut self, via: V, tags: Option<Vec<String>>) -> &mut Path {
        let tags:Vec<String> = if let Some(t) = tags { t } else { Vec::new() };
        self.path.out_with_tags(tags, via.into());
        self
    }


    // ///////////////////////////
    // // Out(path: Path, tags: String[])
    // ///////////////////////////
    // pub fn out_path(self, path: &Path, tags: Option<Vec<String>>) -> Path {
    //     self._in_out_path(path, tags, false)
    // }


    pub fn both<V: Into<path::Via>>(&mut self, via: V, tags: Option<Vec<String>>) -> &mut Path {
        let tags:Vec<String> = if let Some(t) = tags { t } else { Vec::new() };
        self.path.both_with_tags(tags, via.into());
        self
    }


    // ///////////////////////////
    // // Both(values: String[], tags: String[])
    // ///////////////////////////
    // pub fn both_values(self, values: Vec<Value>, tags: Option<Vec<String>>) -> Path {
    //     let tags:Vec<String> = if let Some(t) = tags { t } else { Vec::new() };
    //     let via = path::Via::Values(values);
        
    //     Path::new(self.session, self.finals, self.path.both_with_tags(tags, via))
    // }


    // ///////////////////////////
    // // Both(path: Path, tags: String[])
    // ///////////////////////////
    // pub fn both_path(self, path: &Path, tags: Option<Vec<String>>) -> Path {
    //     let tags:Vec<String> = if let Some(t) = tags { t } else { Vec::new() };
    //     let via = path::Via::Path(path.path.clone());
        
    //     Path::new(self.session, self.finals, self.path.both_with_tags(tags, via))
    // }


    ///////////////////////////
    // Follow(path: Path)
    ///////////////////////////
    pub fn follow(&mut self, ep: &Path) -> &mut Path {
        self.path.follow(ep.path.clone());
        self
    }


    ///////////////////////////
    // FollowR(path: Path)
    ///////////////////////////
    pub fn follow_r(&mut self, ep: &Path) -> &mut Path {
        self.path.follow_reverse(ep.path.clone());
        self
    }


    ///////////////////////////
    // FollowRecursive(path: Path, maxDepth: int, tags: Stringp[])
    ///////////////////////////
    pub fn follow_recursive_path(&mut self, path: &Path, max_depth: Option<i32>, tags: Option<Vec<String>>) -> &mut Path {
        let tags:Vec<String> = if let Some(t) = tags { t } else { Vec::new() };
        let via = path::Via::Path(path.path.clone());
        let max_depth = match max_depth { Some(d) => d, None => 50 };
        self.path.follow_recursive(via, max_depth, tags);
        self
    }


    ///////////////////////////
    // FollowRecursive(value: String, maxDepth: int, tags: Stringp[])
    ///////////////////////////
    pub fn follow_recursive_value(&mut self, value: Value, max_depth: Option<i32>, tags: Option<Vec<String>>) -> &mut Path {
        let tags:Vec<String> = if let Some(t) = tags { t } else { Vec::new() };
        let via = path::Via::Values(vec![value]);
        let max_depth = match max_depth { Some(d) => d, None => 50 };
        self.path.follow_recursive(via, max_depth, tags);
        self
    }


    ///////////////////////////
    // And(path: Path)
    // Intersect(path: Path)
    ///////////////////////////
    pub fn intersect(&mut self, path: &Path) -> &mut Path {
        self.path.and(path.path.clone());
        self
    }


    ///////////////////////////
    // Or(path: Path)
    // Union(path: Path)
    ///////////////////////////
    pub fn union(&mut self, path: &Path) -> &mut Path {
        self.path.or(path.path.clone());
        self
    }


    ///////////////////////////
    // Back(tag: String)
    ///////////////////////////
    pub fn back(&mut self, tag: String) -> &mut Path {
        self
    }

    ///////////////////////////
    // Back(tags: String[])
    ///////////////////////////
    pub fn tag(&mut self, tags: Vec<String>) -> &mut Path {
        self
    }

    ///////////////////////////
    // As(tags: String[])
    ///////////////////////////
    pub fn r#as(&mut self, tags: Vec<String>) -> &mut Path {
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
    pub fn has(&mut self, predicate: String, object: String) -> &mut Path {
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
    pub fn has_r(&mut self, predicate: String, object: String) -> &mut Path {
        self
    }


    ///////////////////////////
    // Save(predicate: String, tag: String)
    ///////////////////////////
    pub fn save(&mut self, predicate: String, tag: String) -> &mut Path {
        self
    }

    ///////////////////////////
    // SaveR(predicate: String, tag: String)
    ///////////////////////////
    pub fn save_r(&mut self, predicate: String, tag: String) -> &mut Path {
        self
    }

    ///////////////////////////
    // SaveOpt(predicate: String, tag: String)
    ///////////////////////////
    pub fn save_opt(&mut self, predicate: String, tag: String) -> &mut Path {
        self
    }

    ///////////////////////////
    // SaveOptR(predicate: String, tag: String)
    ///////////////////////////
    pub fn save_opt_r(&mut self, predicate: String, tag: String) -> &mut Path {
        self
    }

    ///////////////////////////
    // Except(path: Path)
    ///////////////////////////
    pub fn except(&mut self, path: &Path) -> &mut Path {
        self
    }

    ///////////////////////////
    // Unique()
    ///////////////////////////
    pub fn unique(&mut self) -> &mut Path {
        self
    }

    ///////////////////////////
    // Difference(path: Path)
    ///////////////////////////
    pub fn difference(&mut self, path: &Path) -> &mut Path {
        self
    }

    ///////////////////////////
    // Labels()
    ///////////////////////////
    pub fn labels(&mut self) -> &mut Path {
        self
    }

    ///////////////////////////
    // InPredicates(tag:String)
    ///////////////////////////
    pub fn in_predicates(&mut self, tag: String) -> &mut Path {
        self
    }

    ///////////////////////////
    // OutPredicates()
    ///////////////////////////
    pub fn out_predicates(&mut self) -> &mut Path {
        self
    }

    ///////////////////////////
    // SaveInPredicates(tag:String)
    ///////////////////////////
    pub fn save_in_predicates(&mut self, tag: String) -> &mut Path {
        self
    }

    ///////////////////////////
    // SaveOutPredicates(tag:String)
    ///////////////////////////
    pub fn save_out_predicates(&mut self, tag: String) -> &mut Path {
        self
    }


    ///////////////////////////
    // LabelContext(values: String[], tags: String[])
    ///////////////////////////
    pub fn label_context_values(&mut self, values: Vec<String>, tags: Vec<String>) -> &mut Path {
        self
    }

    ///////////////////////////
    // LabelContext(path: Path, tags: String[])
    ///////////////////////////
    pub fn label_context_path(&mut self, path: &Path, tags: Vec<String>) -> &mut Path {
        self
    }


    ///////////////////////////
    // Filter(filter: Filter)
    ///////////////////////////
    pub fn filter<F: Into<ValueFilters>>(&mut self, filters: F) -> &mut Path {
        self.path.filters(filters.into().filters);
        self
    }

    ///////////////////////////
    // Limit(limit: Number)
    ///////////////////////////
    pub fn limit(&mut self, limit: i32) -> &mut Path {
        self
    }

    ///////////////////////////
    // Skip(offset: Number)
    ///////////////////////////
    pub fn skip(&mut self, offset: i32) -> &mut Path {
        self
    }

    ///////////////////////////
    // Order()
    ///////////////////////////
    pub fn order(&mut self) -> &mut Path {
        self
    }


}



pub struct ValueFilters {
    filters: Vec<Rc<dyn shape::ValueFilter>>
}

impl From<Rc<dyn shape::ValueFilter>> for ValueFilters {
    fn from(f: Rc<dyn shape::ValueFilter>) -> ValueFilters {
        ValueFilters {
            filters: vec![f]
        }
    }
}

impl From<Vec<Rc<dyn shape::ValueFilter>>> for ValueFilters {
    fn from(f: Vec<Rc<dyn shape::ValueFilter>>) -> ValueFilters {
        ValueFilters {
            filters: f
        }
    }
}



pub fn lt<V: Into<Value>>(v: V) -> Rc<dyn shape::ValueFilter> {
    Rc::new(shape::Comparison::new(iterator::value_filter::Operator::LT, v.into()))
}

pub fn lte<V: Into<Value>>(v: V) -> Rc<dyn shape::ValueFilter> {
    Rc::new(shape::Comparison::new(iterator::value_filter::Operator::LTE, v.into()))
}

pub fn gt<V: Into<Value>>(v: V) -> Rc<dyn shape::ValueFilter> {
    Rc::new(shape::Comparison::new(iterator::value_filter::Operator::GT, v.into()))
}

pub fn gte<V: Into<Value>>(v: V) -> Rc<dyn shape::ValueFilter> {
    Rc::new(shape::Comparison::new(iterator::value_filter::Operator::GTE, v.into()))
}

pub fn regex<S: Into<String>>(pattern: S) -> Rc<dyn shape::ValueFilter> {
    Rc::new(shape::Regexp::new(pattern.into()))
}

pub fn like<S: Into<String>>(pattern: S) -> Rc<dyn shape::ValueFilter> {
    Rc::new(shape::Wildcard::new(pattern.into()))
}