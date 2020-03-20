use wasm_bindgen::prelude::*;
use super::path;
use std::rc::Rc;
use std::cell::RefCell;
use crate::graph::quad::{QuadStore, QuadWriter, IgnoreOptions, Quad};
use crate::graph::graphmock;
use crate::graph::value::Value;

use super::gizmo;

#[wasm_bindgen(js_name = NewMemoryGraph)]
pub fn new_memory_graph() -> Store {
    let store = gizmo::new_memory_graph();
    Store {
        store, 
        graph: Graph::new()
    }
}


#[wasm_bindgen]
pub struct Store {
    store: gizmo::Store,
    graph: Graph
}


#[wasm_bindgen]
impl Store {

    fn _graph(&self) -> gizmo::Graph {
        self.store.graph()
    }

    pub fn write(&self, jquads: Box<[JsValue]>) {

        let mut quads:Vec<Quad> = Vec::new();
        for jq in &*jquads {
            let quad:Quad = jq.into_serde().unwrap();
            quads.push(quad);
        }

        self.store.write(quads)
    }

    pub fn read(&self) -> JsValue {
        JsValue::from_serde(&self.store.read()).unwrap()
    }

    pub fn delete(&self, quads: Box<[JsValue]>) {

    }

    pub fn graph(self) -> Graph {
        return self.graph;
    }

    pub fn g(self) -> Graph {
        return self.graph;
    }
}


#[wasm_bindgen]
pub struct Graph {
    graph: gizmo::Graph,
    path: Option<Path>
}


#[wasm_bindgen]
impl Graph {

    fn new(graph: gizmo::Graph) -> Graph {
        Graph {
            graph,
            path: None
        }
    }

    // Array of string node_ids
    #[wasm_bindgen(js_name = V)] 
    pub fn v(mut self, qv: Option<Box<[JsValue]>>) -> Path {
        self.path = Some(Path::new(self.graph.v(Some(Vec::new()))));
        self.path.unwrap()
    }

    #[wasm_bindgen(js_name = M)]
    pub fn m(mut self) -> Path {
        self.path = Some(Path::new(self.graph.m()));
        self.path.unwrap()
    }
}


#[wasm_bindgen]
#[derive(Clone)]
pub struct Path {
    path: gizmo::Path
}

#[wasm_bindgen]
impl Path {
    fn new(path: gizmo::Path) -> Path {
        Path {
            path
        }
    }

    ///////////////
    // Finals
    ///////////////

    // //#[wasm_bindgen(js_name = getLimit)]
    // pub fn get_limit(self, limit: &JsValue) -> JsValue {
    //     JsValue::NULL
    // }

    // pub fn all(self) -> JsValue {
    //     JsValue::NULL
    // }

    // //#[wasm_bindgen(js_name = toArray)]
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
    
    fn box_jsvalue_to_vec_value(js_values: Box<[JsValue]>) -> Vec<Value> {
        let mut v = Vec::new();
        for i in 0..js_values.len() {
            v.push(js_values.get(i).unwrap().into());
        }
        v
    }

    fn box_jsvalue_to_vec_strings(js_values: Box<[JsValue]>) -> Vec<String> {
        let mut v = Vec::new();
        for i in 0..js_values.len() {
            if let Some(s) = js_values.get(i).unwrap().as_string() {
               v.push(s);
            }
        }
        v
    }


    ///////////////////////////
    // Is(nodes: String[])
    ///////////////////////////
    pub fn is(self, arr: Box<[JsValue]>) -> Path {
        let nodes = Path::box_jsvalue_to_vec_value(arr);
        let np = self.path.is(nodes);
        Path::new(np)
    }


    ///////////////////////////
    // In(values: String[], tags: String[])
    ///////////////////////////
    pub fn in_values(self, values: Box<[JsValue]>, tags: Option<Box<[JsValue]>>) -> Path {
        let tags = match tags { Some(t) => Some(Path::box_jsvalue_to_vec_strings(t)), None => None };
        let values = Path::box_jsvalue_to_vec_value(values);
        let np = self.path.in_values(values, tags);
        Path::new(np)
    }


    ///////////////////////////
    // In(path: Path, tags: String[])
    ///////////////////////////
    pub fn in_path(self, path: &Path, tags: Option<Box<[JsValue]>>) -> Path {
        let tags = match tags { Some(t) => Some(Path::box_jsvalue_to_vec_strings(t)), None => None };
        let path = path.path;
        let np = self.path.in_path(&path, tags);
        Path::new(np)
    }


    ///////////////////////////
    // Out(values: String[], tags: String[])
    ///////////////////////////
    pub fn out_values(self, values: Box<[JsValue]>, tags: Option<Box<[JsValue]>>) -> Path {
        let tags = match tags { Some(t) => Some(Path::box_jsvalue_to_vec_strings(t)), None => None };
        let values = Path::box_jsvalue_to_vec_value(values);
        let np = self.path.out_values(values, tags);
        Path::new(np)
    }


    ///////////////////////////
    // Out(path: Path, tags: String[])
    ///////////////////////////
    pub fn out_path(self, path: &Path, tags: Option<Box<[JsValue]>>) -> Path {
        let tags = match tags { Some(t) => Some(Path::box_jsvalue_to_vec_strings(t)), None => None };
        let path = path.path;
        let np = self.path.out_path(&path, tags);
        Path::new(np)
    }


    ///////////////////////////
    // Both(values: String[], tags: String[])
    ///////////////////////////
    pub fn both_values(self, values: Box<[JsValue]>, tags: Option<Box<[JsValue]>>) -> Path {
        let tags = match tags { Some(t) => Some(Path::box_jsvalue_to_vec_strings(t)), None => None };
        let values = Path::box_jsvalue_to_vec_value(values);
        let np = self.path.both_values(values, tags);
        Path::new(np)
    }


    ///////////////////////////
    // Both(path: Path, tags: String[])
    ///////////////////////////
    pub fn both_path(self, path: &Path, tags: Option<Box<[JsValue]>>) -> Path {
        let tags = match tags { Some(t) => Some(Path::box_jsvalue_to_vec_strings(t)), None => None };
        let path = path.path;
        let np = self.path.both_path(&path, tags);
        Path::new(np)
    }


    ///////////////////////////
    // Follow(path: Path)
    ///////////////////////////
    pub fn follow(self, ep: &Path) -> Path {
        let path = &ep.path;
        let np = self.path.follow(&path);
        Path::new(np)
    }


    ///////////////////////////
    // FollowR(path: Path)
    ///////////////////////////
    pub fn follow_r(self, ep: &Path) -> Path {
        let path = &ep.path;
        let np = self.path.follow_r(&path);
        Path::new(np)
    }


    ///////////////////////////
    // FollowRecursive(path: Path, maxDepth: int, tags: Stringp[])
    ///////////////////////////
    pub fn follow_recursive_path(self, path: &Path, max_depth: Option<i32>, tags: Option<Box<[JsValue]>>) -> Path {
        let tags = match tags { Some(t) => Some(Path::box_jsvalue_to_vec_strings(t)), None => None };
        let path = path.path;
        let np = self.path.follow_recursive_path(&path, max_depth, tags);
        Path::new(np)
    }


    ///////////////////////////
    // FollowRecursive(value: String, maxDepth: int, tags: Stringp[])
    ///////////////////////////
    pub fn follow_recursive_value(self, value: &JsValue, max_depth: Option<i32>, tags: Option<Box<[JsValue]>>) -> Path {
        let tags = match tags { Some(t) => Some(Path::box_jsvalue_to_vec_strings(t)), None => None };
        let value = value.into();
        let np = self.path.follow_recursive_value(value, max_depth, tags);
        Path::new(np)
    }


    ///////////////////////////
    // And(path: Path)
    // Intersect(path: Path)
    ///////////////////////////
    pub fn intersect(self, path: &Path) -> Path {
        let path = &path.path;
        let np = self.path.intersect(&path);
        Path::new(np)
    }


    ///////////////////////////
    // Or(path: Path)
    // Union(path: Path)
    ///////////////////////////
    pub fn union(self, path: &Path) -> Path {
        let path = &path.path;
        let np = self.path.union(&path);
        Path::new(np)
    }


    // ///////////////////////////
    // // Back(tag: String)
    // ///////////////////////////
    // pub fn back(self, tag: &JsValue) -> Path {
    //     self
    // }

    // ///////////////////////////
    // // Back(tags: String[])
    // ///////////////////////////
    // pub fn tag(self, tags: Box<[JsValue]>) -> Path {
    //     self
    // }

    // ///////////////////////////
    // // As(tags: String[])
    // ///////////////////////////
    // #[wasm_bindgen(js_name = as)]
    // pub fn r#as(self, tags: Box<[JsValue]>) -> Path {
    //     self
    // }

    // ///////////////////////////
    // // Has(predicate: String, object: String)
    // ///////////////////////////

    // ///////////////////////////
    // // *Has(predicate: Path, object: String)
    // // *Has(predicate: String, filters: Filter[])
    // // *Has(predicate: Path, filters: Filter[])
    // ///////////////////////////
    // pub fn has(self, predicate: &JsValue, object: &JsValue) -> Path {
    //     self
    // }

    // ///////////////////////////
    // // HasR(predicate: String, object: String)
    // ///////////////////////////
    
    // ///////////////////////////
    // // *HasR(predicate: Path, object: String)
    // // *HasR(predicate: String, filters: Filter[])
    // // *HasR(predicate: Path, filters: Filter[])
    // ///////////////////////////
    // pub fn has_r(self, args: Option<Box<[JsValue]>>) -> Path {
    //     self
    // }


    // ///////////////////////////
    // // Save(predicate: String, tag: String)
    // ///////////////////////////
    // pub fn save(self, predicate: &JsValue, object: &JsValue) -> Path {
    //     self
    // }

    // ///////////////////////////
    // // SaveR(predicate: String, tag: String)
    // ///////////////////////////
    // pub fn save_r(self, args: Option<Box<[JsValue]>>) -> Path {
    //     self
    // }

    // ///////////////////////////
    // // SaveOpt(predicate: String, tag: String)
    // ///////////////////////////
    // pub fn save_opt(self, args: Option<Box<[JsValue]>>) -> Path {
    //     self
    // }

    // ///////////////////////////
    // // SaveOptR(predicate: String, tag: String)
    // ///////////////////////////
    // pub fn save_opt_r(self, args: Option<Box<[JsValue]>>) -> Path {
    //     self
    // }

    // ///////////////////////////
    // // Except(path: Path)
    // ///////////////////////////
    // pub fn except(self, path: &JsValue) -> Path {
    //     self
    // }

    // ///////////////////////////
    // // Unique()
    // ///////////////////////////
    // pub fn unique(self) -> Path {
    //     self
    // }

    // ///////////////////////////
    // // Difference(path: Path)
    // ///////////////////////////
    // pub fn difference(self, path: &JsValue) -> Path {
    //     self
    // }

    // ///////////////////////////
    // // Labels()
    // ///////////////////////////
    // pub fn labels(self) -> Path {
    //     self
    // }

    // ///////////////////////////
    // // InPredicates(tag:String)
    // ///////////////////////////
    // pub fn in_predicates(self, tag: &JsValue) -> Path {
    //     self
    // }

    // ///////////////////////////
    // // OutPredicates()
    // ///////////////////////////
    // pub fn out_predicates(self, tag: &JsValue) -> Path {
    //     self
    // }

    // ///////////////////////////
    // // SaveInPredicates(tag:String)
    // ///////////////////////////
    // pub fn save_in_predicates(self, tag: &JsValue) -> Path {
    //     self
    // }

    // ///////////////////////////
    // // SaveOutPredicates(tag:String)
    // ///////////////////////////
    // pub fn save_out_predicates(self, tag: &JsValue) -> Path {
    //     self
    // }


    // ///////////////////////////
    // // LabelContext(values: String[], tags: String[])
    // ///////////////////////////
    // pub fn label_context_values(self, predicate_path: Option<Box<[JsValue]>>, tags: Option<Box<[JsValue]>>) -> Path {
    //     self
    // }

    // ///////////////////////////
    // // LabelContext(path: Path, tags: String[])
    // ///////////////////////////
    // pub fn label_context_path(self, predicate_path: Option<Box<[JsValue]>>, tags: Option<Box<[JsValue]>>) -> Path {
    //     self
    // }


    // ///////////////////////////
    // // Filter(filter: Filter)
    // ///////////////////////////
    // pub fn filter(self, args: Option<Box<[JsValue]>>) -> Path {
    //     self
    // }

    // ///////////////////////////
    // // Limit(limit: Number)
    // ///////////////////////////
    // pub fn limit(self, limit: &JsValue) -> Path {
    //     self
    // }

    // ///////////////////////////
    // // Skip(offset: Number)
    // ///////////////////////////
    // pub fn skip(self, offset: &JsValue) -> Path {
    //     self
    // }

    // ///////////////////////////
    // // Order()
    // ///////////////////////////
    // pub fn order(self) -> Path {
    //     self
    // }


}



