use wasm_bindgen::prelude::*;
use super::path;
use std::rc::Rc;
use std::cell::RefCell;
use crate::graph::quad::QuadStore;
use crate::graph::graphmock;
use crate::graph::value::Value;

#[wasm_bindgen(js_name = NewMemoryGraph)]
pub fn new_memory_graph() -> Store {
    let qs = Rc::new(RefCell::new(graphmock::Store::new()));
    Store {
        g: Graph::new(qs.clone()),
        qs
    }
}


#[wasm_bindgen]
pub struct Store {
    g: Graph,
    qs: Rc<RefCell<dyn QuadStore>>
}


#[wasm_bindgen]
impl Store {
    pub fn graph(self) -> Graph {
        return self.g;
    }

    pub fn g(self) -> Graph {
        return self.g;
    }
}


#[wasm_bindgen]
pub struct Graph {
    path: Option<Path>,
    qs: Rc<RefCell<dyn QuadStore>>
}


#[wasm_bindgen]
impl Graph {

    fn new(qs: Rc<RefCell<dyn QuadStore>>) -> Graph {
        Graph {
            path: None,
            qs
        }
    }

    // Array of string node_ids
    #[wasm_bindgen(js_name = V)] 
    pub fn v(mut self, qv: Option<Box<[JsValue]>>) -> Path {

        self.path = Some(Path::new(true, path::Path::new(None, Vec::new())));

        return self.path.unwrap();
    }

    #[wasm_bindgen(js_name = M)]
    pub fn m(mut self) -> Path {

        self.path = Some(Path::new(false, path::Path::new(None, Vec::new())));
        
        return self.path.unwrap();
    }
}


#[wasm_bindgen]
#[derive(Clone)]
pub struct Path {
    finals: bool,
    path: path::Path
}

#[wasm_bindgen]
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
    
    // #[wasm_bindgen]
    // pub fn all(self, args: Box<[JsValue]>) {
    //     for i in 0..args.len() {
    //         console::log_1(&args[i]);
    //     }
    // }


    ///////////////
    // Finals
    ///////////////

    //#[wasm_bindgen(js_name = getLimit)]
    pub fn get_limit(self, limit: &JsValue) -> JsValue {
        JsValue::NULL
    }

    pub fn all(self) -> JsValue {
        JsValue::NULL
    }

    //#[wasm_bindgen(js_name = toArray)]
    pub fn to_array(self, args: Option<Box<[JsValue]>>) -> JsValue {
        JsValue::NULL
    }

    //#[wasm_bindgen(js_name = tagArray)]
    pub fn tag_array(self, args: Option<Box<[JsValue]>>) -> JsValue {
        JsValue::NULL
    }

    //#[wasm_bindgen(js_name = toValue)]
    pub fn to_value(self) -> JsValue {
        JsValue::NULL
    }

    //#[wasm_bindgen(js_name = tagValue)]
    pub fn tag_value(self) -> JsValue {
        JsValue::NULL
    }

    pub fn map(self, callback: &JsValue) {

    }

    //#[wasm_bindgen(js_name = forEach)]
    pub fn for_each(self, callback: &JsValue) {

    }

    pub fn count(self) -> JsValue {
        JsValue::NULL
    }




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
        Path::new(self.finals, np)
    }



    fn _in_out_values(self, values: Box<[JsValue]>, tags: Option<Box<[JsValue]>>, r#in: bool) -> Path {
        let tags:Vec<String> = if let Some(t) = tags { Path::box_jsvalue_to_vec_strings(t) } else { Vec::new() };
        let via = path::Via::Values(Path::box_jsvalue_to_vec_value(values));

        let np = if r#in { self.path.in_with_tags(tags, via) } else { self.path.out_with_tags(tags, via) };
        
        Path::new(self.finals, np)
    }

    fn _in_out_path(self, path: &Path, tags: Option<Box<[JsValue]>>, r#in: bool) -> Path {
        let tags:Vec<String> = if let Some(t) = tags { Path::box_jsvalue_to_vec_strings(t) } else { Vec::new() };
        let via = path::Via::Path(path.path.clone());

        let np = if r#in { self.path.in_with_tags(tags, via) } else { self.path.out_with_tags(tags, via) };
        
        Path::new(self.finals, np)
    }

    ///////////////////////////
    // In(values: String[], tags: String[])
    ///////////////////////////
    pub fn in_values(self, values: Box<[JsValue]>, tags: Option<Box<[JsValue]>>) -> Path {
        self._in_out_values(values, tags, true)
    }

    ///////////////////////////
    // In(path: Path, tags: String[])
    ///////////////////////////
    pub fn in_path(self, path: &Path, tags: Option<Box<[JsValue]>>) -> Path {
        self._in_out_path(path, tags, true)
    }

    ///////////////////////////
    // Out(values: String[], tags: String[])
    ///////////////////////////
    pub fn out_values(self, values: Box<[JsValue]>, tags: Option<Box<[JsValue]>>) -> Path {
        self._in_out_values(values, tags, false)
    }

    ///////////////////////////
    // Out(path: Path, tags: String[])
    ///////////////////////////
    pub fn out_path(self, path: &Path, tags: Option<Box<[JsValue]>>) -> Path {
        self._in_out_path(path, tags, false)
    }




    ///////////////////////////
    // Both(values: String[], tags: String[])
    ///////////////////////////
    pub fn both_values(self, values: Box<[JsValue]>, tags: Option<Box<[JsValue]>>, r#in: bool) -> Path {
        let tags:Vec<String> = if let Some(t) = tags { Path::box_jsvalue_to_vec_strings(t) } else { Vec::new() };
        let via = path::Via::Values(Path::box_jsvalue_to_vec_value(values));
        
        Path::new(self.finals, self.path.both_with_tags(tags, via))
    }

    ///////////////////////////
    // Both(path: Path, tags: String[])
    ///////////////////////////
    pub fn both_path(self, path: &Path, tags: Option<Box<[JsValue]>>, r#in: bool) -> Path {
        let tags:Vec<String> = if let Some(t) = tags { Path::box_jsvalue_to_vec_strings(t) } else { Vec::new() };
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
    pub fn follow_recursive(self, path: &Path) -> Path {
        self
    }

    ///////////////////////////
    // And(path: Path)
    ///////////////////////////
    pub fn and(self, path: &JsValue) -> Path {
        self
    }

    ///////////////////////////
    // Intersect(path: Path)
    ///////////////////////////
    pub fn intersect(self, path: &JsValue) -> Path {
        self
    }

    ///////////////////////////
    // Union(path: Path)
    ///////////////////////////
    pub fn union(self, path: &JsValue) -> Path {
        self
    }

    ///////////////////////////
    // Or(path: Path)
    ///////////////////////////
    pub fn or(self, path: &JsValue) -> Path {
        self
    }

    ///////////////////////////
    // Back(tag: String)
    ///////////////////////////
    pub fn back(self, tag: &JsValue) -> Path {
        self
    }

    ///////////////////////////
    // Back(tags: String[])
    ///////////////////////////
    pub fn tag(self, tags: Box<[JsValue]>) -> Path {
        self
    }

    ///////////////////////////
    // As(tags: String[])
    ///////////////////////////
    #[wasm_bindgen(js_name = as)]
    pub fn r#as(self, tags: Box<[JsValue]>) -> Path {
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
    pub fn has(self, predicate: &JsValue, object: &JsValue) -> Path {
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
    pub fn has_r(self, args: Option<Box<[JsValue]>>) -> Path {
        self
    }


    ///////////////////////////
    // Save(predicate: String, tag: String)
    ///////////////////////////
    pub fn save(self, predicate: &JsValue, object: &JsValue) -> Path {
        self
    }

    ///////////////////////////
    // SaveR(predicate: String, tag: String)
    ///////////////////////////
    pub fn save_r(self, args: Option<Box<[JsValue]>>) -> Path {
        self
    }

    ///////////////////////////
    // SaveOpt(predicate: String, tag: String)
    ///////////////////////////
    pub fn save_opt(self, args: Option<Box<[JsValue]>>) -> Path {
        self
    }

    ///////////////////////////
    // SaveOptR(predicate: String, tag: String)
    ///////////////////////////
    pub fn save_opt_r(self, args: Option<Box<[JsValue]>>) -> Path {
        self
    }

    ///////////////////////////
    // Except(path: Path)
    ///////////////////////////
    pub fn except(self, path: &JsValue) -> Path {
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
    pub fn difference(self, path: &JsValue) -> Path {
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
    pub fn in_predicates(self, tag: &JsValue) -> Path {
        self
    }

    ///////////////////////////
    // OutPredicates()
    ///////////////////////////
    pub fn out_predicates(self, tag: &JsValue) -> Path {
        self
    }

    ///////////////////////////
    // SaveInPredicates(tag:String)
    ///////////////////////////
    pub fn save_in_predicates(self, tag: &JsValue) -> Path {
        self
    }

    ///////////////////////////
    // SaveOutPredicates(tag:String)
    ///////////////////////////
    pub fn save_out_predicates(self, tag: &JsValue) -> Path {
        self
    }


    ///////////////////////////
    // LabelContext(values: String[], tags: String[])
    ///////////////////////////
    pub fn label_context_values(self, predicate_path: Option<Box<[JsValue]>>, tags: Option<Box<[JsValue]>>) -> Path {
        self
    }

    ///////////////////////////
    // LabelContext(path: Path, tags: String[])
    ///////////////////////////
    pub fn label_context_path(self, predicate_path: Option<Box<[JsValue]>>, tags: Option<Box<[JsValue]>>) -> Path {
        self
    }


    ///////////////////////////
    // Filter(filter: Filter)
    ///////////////////////////
    pub fn filter(self, args: Option<Box<[JsValue]>>) -> Path {
        self
    }

    ///////////////////////////
    // Limit(limit: Number)
    ///////////////////////////
    pub fn limit(self, limit: &JsValue) -> Path {
        self
    }

    ///////////////////////////
    // Skip(offset: Number)
    ///////////////////////////
    pub fn skip(self, offset: &JsValue) -> Path {
        self
    }

    ///////////////////////////
    // Order()
    ///////////////////////////
    pub fn order(self) -> Path {
        self
    }


}



