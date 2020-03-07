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


    pub fn is(self, arr: Box<[JsValue]>) -> Path {
        let nodes = Path::box_jsvalue_to_vec_value(arr);
        let np = self.path.is(nodes);
        Path::new(self.finals, np)
    }





    pub fn in_out_value(self, value: &JsValue, tags: Option<Box<[JsValue]>>, r#in: bool) -> Path {
        let tags:Vec<String> = if let Some(t) = tags { Path::box_jsvalue_to_vec_strings(t) } else { Vec::new() };
        let via = path::Via::Values(vec![value.into()]);

        let np = if r#in { self.path.in_with_tags(tags, via) } else { self.path.out_with_tags(tags, via) };

        Path::new(self.finals, np)
    }

    pub fn in_out_value_arr(self, values: Box<[JsValue]>, tags: Option<Box<[JsValue]>>, r#in: bool) -> Path {
        let tags:Vec<String> = if let Some(t) = tags { Path::box_jsvalue_to_vec_strings(t) } else { Vec::new() };
        let via = path::Via::Values(Path::box_jsvalue_to_vec_value(values));

        let np = if r#in { self.path.in_with_tags(tags, via) } else { self.path.out_with_tags(tags, via) };
        
        Path::new(self.finals, np)
    }

    pub fn in_out_path(self, path: &Path, tags: Option<Box<[JsValue]>>, r#in: bool) -> Path {
        let tags:Vec<String> = if let Some(t) = tags { Path::box_jsvalue_to_vec_strings(t) } else { Vec::new() };
        let via = path::Via::Path(path.path.clone());

        let np = if r#in { self.path.in_with_tags(tags, via) } else { self.path.out_with_tags(tags, via) };
        
        Path::new(self.finals, np)
    }


    pub fn in_value(self, value: &JsValue, tags: Option<Box<[JsValue]>>) -> Path {
        self.in_out_value(value, tags, true)
    }

    pub fn in_value_arr(self, values: Box<[JsValue]>, tags: Option<Box<[JsValue]>>) -> Path {
        self.in_out_value_arr(values, tags, true)
    }

    pub fn in_path(self, path: &Path, tags: Option<Box<[JsValue]>>) -> Path {
        self.in_out_path(path, tags, true)
    }


    pub fn out_value(self, value: &JsValue, tags: Option<Box<[JsValue]>>) -> Path {
        self.in_out_value(value, tags, false)
    }

    pub fn out_value_arr(self, values: Box<[JsValue]>, tags: Option<Box<[JsValue]>>) -> Path {
        self.in_out_value_arr(values, tags, false)
    }

    pub fn out_path(self, path: &Path, tags: Option<Box<[JsValue]>>) -> Path {
        self.in_out_path(path, tags, false)
    }





    pub fn both(self, predicate_path: &JsValue, tags: Option<Box<[JsValue]>>) -> Path {
        self
    }

    pub fn follow(self, path: &JsValue) -> Path {
        self
    }

    //#[wasm_bindgen(js_name = followR)]
    pub fn follow_r(self, path: &JsValue) -> Path {
        self
    }

    //#[wasm_bindgen(js_name = followRecursive)]
    pub fn follow_recursive(self, args: Option<Box<[JsValue]>>) -> Path {
        self
    }

    pub fn and(self, path: &JsValue) -> Path {
        self
    }

    pub fn itersect(self, path: &JsValue) -> Path {
        self
    }

    pub fn union(self, path: &JsValue) -> Path {
        self
    }

    pub fn or(self, path: &JsValue) -> Path {
        self
    }

    pub fn back(self, tag: &JsValue) -> Path {
        self
    }

    pub fn tag(self, tags: Box<[JsValue]>) -> Path {
        self
    }

    #[wasm_bindgen(js_name = as)]
    pub fn r#as(self, tags: Box<[JsValue]>) -> Path {
        self
    }

    pub fn has(self, predicate: &JsValue, object: &JsValue) -> Path {
        self
    }

    //#[wasm_bindgen(js_name = hasR)]
    pub fn has_r(self, args: Option<Box<[JsValue]>>) -> Path {
        self
    }

    pub fn save(self, predicate: &JsValue, object: &JsValue) -> Path {
        self
    }

    //#[wasm_bindgen(js_name = saveR)]
    pub fn save_r(self, args: Option<Box<[JsValue]>>) -> Path {
        self
    }

    //#[wasm_bindgen(js_name = saveOpt)]
    pub fn save_opt(self, args: Option<Box<[JsValue]>>) -> Path {
        self
    }

    //#[wasm_bindgen(js_name = saveOptR)]
    pub fn save_opt_r(self, args: Option<Box<[JsValue]>>) -> Path {
        self
    }

    pub fn except(self, path: &JsValue) -> Path {
        self
    }

    pub fn unique(self) -> Path {
        self
    }

    pub fn difference(self, path: &JsValue) -> Path {
        self
    }

    pub fn labels(self) -> Path {
        self
    }

    //#[wasm_bindgen(js_name = inPredicates)]
    pub fn in_predicates(self, tag: &JsValue) -> Path {
        self
    }

    //#[wasm_bindgen(js_name = outPredicates)]
    pub fn out_predicates(self, tag: &JsValue) -> Path {
        self
    }

    //#[wasm_bindgen(js_name = saveInPredicates)]
    pub fn save_in_predicates(self, tag: &JsValue) -> Path {
        self
    }

    //#[wasm_bindgen(js_name = saveOutPredicates)]
    pub fn save_out_predicates(self, tag: &JsValue) -> Path {
        self
    }

    //#[wasm_bindgen(js_name = labelContext)]
    pub fn label_context(self, predicate_path: Option<Box<[JsValue]>>, tags: Option<Box<[JsValue]>>) -> Path {
        self
    }

    pub fn filter(self, args: Option<Box<[JsValue]>>) -> Path {
        self
    }

    pub fn limit(self, limit: &JsValue) -> Path {
        self
    }

    pub fn skip(self, offset: &JsValue) -> Path {
        self
    }

    pub fn order(self) -> Path {
        self
    }

    fn in_out(self, predicate: &JsValue, tags: Option<Box<[JsValue]>>, in_: bool) -> Path {
        // if let Some(s) = v {
        //     console::log_1(&JsValue::from("Some"));
        //     for i in 0..s.len() {
        //         console::log_1(&s[i]);
        //     }
        // } else {
        //     console::log_1(&JsValue::from("None"));
        // }
        self
    }
}



