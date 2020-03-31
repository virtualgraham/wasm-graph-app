use super::path;
use super::shape;
use std::rc::Rc;
use std::cell::RefCell;
use crate::graph::quad::{QuadStore, QuadWriter, IgnoreOptions, Quad};
use crate::graph::graphmock;
use crate::graph::memstore;
use crate::graph::value::Value;
use crate::graph::iterator;
use io_context::Context;
use std::collections::HashMap;
use crate::graph::refs::Ref;


pub fn new_memory_graph() -> GraphWrapper {
    let qs = Rc::new(RefCell::new(memstore::quadstore::MemStore::new()));
    //let qs = Rc::new(RefCell::new(graphmock::Store::new()));

    let s = Rc::new(RefCell::new(Session {
        ctx: Rc::new(RefCell::new(Context::background())),
        qs: qs.clone(),
        qw: QuadWriter::new(qs.clone(), IgnoreOptions{ignore_dup: true, ignore_missing: true})
    }));

    let g = Graph::new(s.clone());

    GraphWrapper {
        graph: g,
        session: s
    }
}


pub struct GraphWrapper {
    pub graph: Graph,
    pub session: Rc<RefCell<Session>>
}


impl GraphWrapper {
    pub fn graph(&self) -> Graph {
        return self.graph.clone();
    }

    pub fn g(&self) -> Graph {
        return self.graph.clone();
    }

    pub fn write(&self, quads: Vec<Quad>) {
        self.session.borrow().write(quads)
    }

    pub fn read(&self) -> Vec<Quad> {
        self.session.borrow().read()
    }

    pub fn delete(&self, quads: Vec<Quad>) {
        self.session.borrow().delete(quads)
    }
}


pub struct Session {
    ctx: Rc<RefCell<Context>>,
    qs: Rc<RefCell<dyn QuadStore>>,
    qw: QuadWriter
}

impl Session {
    fn write(&self, quads: Vec<Quad>) {
        for quad in &quads {
            self.qw.add_quad(quad.clone()).unwrap();
        }
    }

    fn read(&self) -> Vec<Quad> {
        // TODO: implement
        vec![Quad::new("a", "b", "c", "d")]
    }

    fn delete(&self, quads: Vec<Quad>) {
        // TODO: implement
    }

    fn run_tag_each_iterator(&mut self, it: Rc<RefCell<dyn iterator::Shape>>) -> iterator::iterate::TagEachIterator {
        iterator::iterate::TagEachIterator::new(self.ctx.clone(), it, false, true)
    }

    fn run_each_iterator(&mut self, it: Rc<RefCell<dyn iterator::Shape>>) -> iterator::iterate::EachIterator {
        iterator::iterate::EachIterator::new(self.ctx.clone(), it, false, true)
    }
}

#[derive(Clone)]
pub struct Graph {
    session: Rc<RefCell<Session>>,
}

impl Graph {
    pub fn new(session: Rc<RefCell<Session>>) -> Graph {
        Graph {
            session
        }
    }

    pub fn v<V: Into<Values>>(&self, qv: V) -> Path {
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
    }

    pub fn m(&self) -> Path {
        Path::new(self.session.clone(), false, path::Path::start_morphism(Vec::new()))
    }
}


#[derive(Clone)]
pub struct Path {
    pub session: Rc<RefCell<Session>>,
    finals: bool,
    pub path: path::Path
}

impl Path {

    fn new(session: Rc<RefCell<Session>>, finals: bool, path: path::Path) -> Path {
        Path {
            session,
            finals,
            path
        }
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

    pub fn iter(&self) -> impl Iterator<Item = HashMap<String, Value>> {
        let it = self.build_iterator_tree();
        let it = iterator::save::tag(&it, &"id");
        let qs = self.session.borrow().qs.clone();
        self.session.borrow_mut().run_tag_each_iterator(it).filter_map(move |r| tags_to_value_map(&r, &*qs.borrow()))
    }

    pub fn iter_values(&self) -> impl Iterator<Item = Value> {
        let it = self.build_iterator_tree();
        let qs = self.session.borrow().qs.clone();
        self.session.borrow_mut().run_each_iterator(it).filter_map(move |r| ref_to_value(&r, &*qs.borrow()))
    }

    pub fn count(&mut self) -> i64 {
        let it = self.build_iterator_tree();
        self.session.borrow_mut().run_each_iterator(it).count() as i64
    }


    ///////////////
    // Traversals
    ///////////////
    
    ///////////////////////////
    // Is(nodes: String[])
    ///////////////////////////
    pub fn is<V: Into<Values>>(&mut self, nodes: V) -> Path {
        self.path.is(nodes.into().to_vec());
        self.clone()
    }

    ///////////////////////////
    // In(values: String[], tags: String[])
    ///////////////////////////
    pub fn r#in<V: Into<path::Via>, T: Into<Tags>>(&mut self, via: V, tags: T) -> Path {
        self.path.in_with_tags(tags.into().to_vec(), via.into());
        self.clone()
    }

    ///////////////////////////
    // Out(values: String[], tags: String[])
    ///////////////////////////
    pub fn out<V: Into<path::Via>, T: Into<Tags>>(&mut self, via: V, tags: T) -> Path {
        self.path.out_with_tags(tags.into().to_vec(), via.into());
        self.clone()
    }

    ///////////////////////////
    // Both(values: String[], tags: String[])
    ///////////////////////////
    pub fn both<V: Into<path::Via>, T: Into<Tags>>(&mut self, via: V, tags: T) -> Path {
        self.path.both_with_tags(tags.into().to_vec(), via.into());
        self.clone()
    }

    ///////////////////////////
    // Follow(path: Path)
    ///////////////////////////
    pub fn follow(&mut self, ep: &Path) -> Path {
        self.path.follow(ep.path.clone());
        self.clone()
    }


    ///////////////////////////
    // FollowR(path: Path)
    ///////////////////////////
    pub fn follow_r(&mut self, ep: &Path) -> Path {
        self.path.follow_reverse(ep.path.clone());
        self.clone()
    }


    ///////////////////////////
    // FollowRecursive(path: Path, maxDepth: int, tags: Stringp[])
    ///////////////////////////
    pub fn follow_recursive_path<T: Into<Tags>>(&mut self, path: &Path, max_depth: Option<i32>, tags: T) -> Path {
        let via = path.into();
        let max_depth = match max_depth { Some(d) => d, None => 50 };
        self.path.follow_recursive(via, max_depth, tags.into().to_vec());
        self.clone()
    }


    ///////////////////////////
    // FollowRecursive(value: String, maxDepth: int, tags: Stringp[])
    ///////////////////////////
    pub fn follow_recursive_value<T: Into<Tags>>(&mut self, value: Value, max_depth: Option<i32>, tags: T) -> Path {
        let via = path::Via::Values(vec![value]);
        let max_depth = match max_depth { Some(d) => d, None => 50 };
        self.path.follow_recursive(via, max_depth, tags.into().to_vec());
        self.clone()
    }


    ///////////////////////////
    // And(path: Path)
    ///////////////////////////
    
    pub fn and(&mut self, path: &Path) -> Path {
        self.intersect(path)
    }


    ///////////////////////////
    // Intersect(path: Path)
    ///////////////////////////

    pub fn intersect(&mut self, path: &Path) -> Path {
        self.path.and(path.path.clone());
        self.clone()
    }


    ///////////////////////////
    // Or(path: Path)
    ///////////////////////////
    
    pub fn or(&mut self, path: &Path) -> Path {
        self.union(path)
    }


    /////////////////////////// 
    // Union(path: Path)
    ///////////////////////////

    pub fn union(&mut self, path: &Path) -> Path {
        self.path.or(path.path.clone());
        self.clone()
    }


    ///////////////////////////
    // Back(tag: String)
    ///////////////////////////
    pub fn back<S: Into<String>>(&mut self, tag: S) -> Path {
        let np = self.path.back(tag.into());
        if let Some(p) = np {
            self.path = p
        }
        self.clone()
    }


    ///////////////////////////
    // Back(tags: String[])
    ///////////////////////////
    pub fn tag<T: Into<Tags>>(&mut self, tags: T) -> Path {
        self.path.tag(tags.into().to_vec());
        self.clone()
    }


    ///////////////////////////
    // As(tags: String[])
    ///////////////////////////
    pub fn r#as<T: Into<Tags>>(&mut self, tags: T) -> Path {
        self.tag(tags)
    }

    
    ///////////////////////////
    // Has(predicate: String, object: String)
    // *Has(predicate: Path, object: String)
    // *Has(predicate: String, filters: Filter[])
    // *Has(predicate: Path, filters: Filter[])
    ///////////////////////////
    pub fn has<V: Into<path::Via>, O: Into<HasObject>>(&mut self, predicate: V, object: O) -> Path {
        match object.into() {
            HasObject::ValueFilters(f) => {
                self.path.has_filter(predicate.into(), false, f.filters);
            },
            HasObject::Values(v) => {
                self.path.has(predicate.into(), false, v.to_vec());
            }
        }
        self.clone()
    }

    ///////////////////////////
    // HasR(predicate: String, object: String)
    // *HasR(predicate: Path, object: String)
    // *HasR(predicate: String, filters: Filter[])
    // *HasR(predicate: Path, filters: Filter[])
    ///////////////////////////
    pub fn has_r<V: Into<path::Via>, O: Into<HasObject>>(&mut self, predicate: V, object: O) -> Path {
        match object.into() {
            HasObject::ValueFilters(f) => {
                self.path.has_filter(predicate.into(), true, f.filters);
            },
            HasObject::Values(v) => {
                self.path.has(predicate.into(), true, v.to_vec());
            }
        }
        self.clone()
    }


    ///////////////////////////
    // Save(predicate: String, tag: String)
    ///////////////////////////
    pub fn save<V: Into<SaveVia>, T: Into<Tag>>(&mut self, via: V, tag: T) -> Path {
        let via:SaveVia = via.into();
        let tag = save_validate(&via, &tag.into());
        self.path.save(via.to_via(), tag, false, false);
        self.clone()
    }

    ///////////////////////////
    // SaveR(predicate: String, tag: String)
    ///////////////////////////
    pub fn save_r<V: Into<SaveVia>, T: Into<Tag>>(&mut self, via: V, tag: T) -> Path {
        let via:SaveVia = via.into();
        let tag = save_validate(&via, &tag.into());
        self.path.save(via.to_via(), tag, true, false);
        self.clone()
    }

    ///////////////////////////
    // SaveOpt(predicate: String, tag: String)
    ///////////////////////////
    pub fn save_opt<V: Into<SaveVia>, T: Into<Tag>>(&mut self, via: V, tag: T) -> Path {
        let via:SaveVia = via.into();
        let tag = save_validate(&via, &tag.into());
        self.path.save(via.to_via(), tag, false, true);
        self.clone()
    }

    ///////////////////////////
    // SaveOptR(predicate: String, tag: String)
    ///////////////////////////
    pub fn save_opt_r<V: Into<SaveVia>, T: Into<Tag>>(&mut self, via: V, tag: T) -> Path {
        let via:SaveVia = via.into();
        let tag = save_validate(&via, &tag.into());
        self.path.save(via.to_via(), tag, true, true);
        self.clone()
    }


    ///////////////////////////
    // Except(path: Path)
    ///////////////////////////
    pub fn except(&mut self, path: &Path) -> Path {
        self.path.except(path.path.clone());
        self.clone()
    }

    ///////////////////////////
    // Unique()
    ///////////////////////////
    pub fn unique(&mut self) -> Path {
        self.path.unique();
        self.clone()
    }

    ///////////////////////////
    // Difference(path: Path)
    ///////////////////////////
    pub fn difference(&mut self, path: &Path) -> Path {
        self.clone()
    }

    ///////////////////////////
    // Labels()
    ///////////////////////////
    pub fn labels(&mut self) -> Path {
        self.path.labels();
        self.clone()
    }

    ///////////////////////////
    // InPredicates(tag:String)
    ///////////////////////////
    pub fn in_predicates(&mut self) -> Path {
        self.path.predicates(true);
        self.clone()
    }

    ///////////////////////////
    // OutPredicates()
    ///////////////////////////
    pub fn out_predicates(&mut self) -> Path {
        self.path.predicates(false);
        self.clone()
    }

    ///////////////////////////
    // SaveInPredicates(tag:String)
    ///////////////////////////
    pub fn save_in_predicates<S: Into<String>>(&mut self, tag: S) -> Path {
        self.path.save_predicates(tag.into(), true);
        self.clone()
    }

    ///////////////////////////
    // SaveOutPredicates(tag:String)
    ///////////////////////////
    pub fn save_out_predicates<S: Into<String>>(&mut self, tag: S) -> Path {
        self.path.save_predicates(tag.into(), false);
        self.clone()
    }


    ///////////////////////////
    // LabelContext(values: String[], tags: String[])
    ///////////////////////////
    pub fn label_context<V: Into<path::Via>, T: Into<Tags>>(&mut self, labels: V, tags: T) -> Path {
        self.path.label_context_with_tags(labels.into(), tags.into().to_vec());
        self.clone()
    }



    ///////////////////////////
    // Filter(filter: Filter)
    ///////////////////////////
    pub fn filter<F: Into<ValueFilters>>(&mut self, filters: F) -> Path {
        self.path.filters(filters.into().filters);
        self.clone()
    }

    ///////////////////////////
    // Limit(limit: Number)
    ///////////////////////////
    pub fn limit(&mut self, limit: i64) -> Path {
        self.path.limit(limit);
        self.clone()
    }

    ///////////////////////////
    // Skip(offset: Number)
    ///////////////////////////
    pub fn skip(&mut self, offset: i64) -> Path {
        self.path.skip(offset);
        self.clone()
    }

    ///////////////////////////
    // Order()
    ///////////////////////////
    pub fn order(&mut self) -> Path {
        self.path.order();
        self.clone()
    }
}

fn save_validate(via: &SaveVia, tag: &Tag) -> String {
    if let SaveVia::Value(v) = via {
        if let Value::None = v {
            panic!("must specify a predicate")
        }
    }

    if let Tag::Some(t) = tag {
        if !t.is_empty() {
            return t.clone();
        } 
    }

    match via {
        SaveVia::Path(_) => panic!("must specify a tag name when saving a path"),
        SaveVia::Value(v) => v.to_string()
    }
}

fn ref_to_value(r: &Ref, qs: &dyn QuadStore) -> Option<Value> {
    qs.name_of(r) 
}

fn tags_to_value_map(m: &HashMap<String, Ref>, qs: &dyn QuadStore) -> Option<HashMap<String, Value>> {
    let mut output_map = HashMap::new();

    for (key, value) in m {
        match qs.name_of(value) {
            Some(v) => { output_map.insert(key.clone(), v); },
            None => {}
        };
    }
    
    if output_map.is_empty() {
        return None
    }

    return Some(output_map)
}




/////////////////////
// Argument Helpers
/////////////////////

pub enum Tags {
    None,
    Some(Vec<String>)
}


impl Tags {
    pub fn to_vec(self) -> Vec<String> {
        match self {
            Tags::None => Vec::new(),
            Tags::Some(v) => v
        }
    }
}

impl From<Option<Vec<String>>> for Tags {
    fn from(v: Option<Vec<String>>) -> Self {
        match v {
            Some(v) => Tags::Some(v),
            None => Tags::None
        }
    }
}

impl From<Vec<&str>> for Tags {
    fn from(v: Vec<&str>) -> Self {
        Tags::Some(v.iter().map(|s| s.to_string()).collect())
    }
}

impl From<&str> for Tags {
    fn from(v: &str) -> Self {
        Tags::Some(vec![v.into()])
    }
}


pub enum Values {
    None,
    Some(Vec<Value>)
}

impl Values {
    pub fn to_vec(self) -> Vec<Value> {
        match self {
            Values::None => Vec::new(),
            Values::Some(v) => v
        }
    }
}

impl From<Option<Value>> for Values {
    fn from(v: Option<Value>) -> Self {
        match v {
            Some(v) => Values::Some(vec![v]),
            None => Values::None
        }
    }
}

impl From<Value> for Values {
    fn from(v: Value) -> Self {
        Values::Some(vec![v])
    }
}

impl From<Vec<Value>> for Values {
    fn from(v: Vec<Value>) -> Self {
        Values::Some(v)
    }
}

impl From<Vec<&str>> for Values {
    fn from(v: Vec<&str>) -> Self {
        Values::Some(v.iter().map(|s| Value::from(*s)).collect())
    }
}

impl From<&str> for Values {
    fn from(v: &str) -> Self {
        Values::Some(vec![v.into()])
    }
}

impl From<String> for Values {
    fn from(v: String) -> Self {
        Values::Some(vec![v.into()])
    }
}


pub enum HasObject {
    ValueFilters(ValueFilters),
    Values(Values)
}

impl From<Rc<dyn shape::ValueFilter>> for HasObject {
    fn from(f: Rc<dyn shape::ValueFilter>) -> Self {
        HasObject::ValueFilters(
            ValueFilters {
                filters: vec![f]
            }
        )
    }
}

impl From<Vec<Rc<dyn shape::ValueFilter>>> for HasObject {
    fn from(f: Vec<Rc<dyn shape::ValueFilter>>) -> Self {
        HasObject::ValueFilters(
            ValueFilters {
                    filters: f
            }
        )
    }
}

impl From<Option<Value>> for HasObject {
    fn from(v: Option<Value>) -> Self {
        HasObject::Values (
            match v {
                Some(v) => Values::Some(vec![v]),
                None => Values::None
            }
        )
    }
}

impl From<Value> for HasObject {
    fn from(v: Value) -> Self {
        HasObject::Values (
            Values::Some(vec![v])
        )
    }
}

impl From<Vec<Value>> for HasObject {
    fn from(v: Vec<Value>) -> Self {
        HasObject::Values (
            Values::Some(v)
        )
    }
}

impl From<Vec<&str>> for HasObject {
    fn from(v: Vec<&str>) -> Self {
        HasObject::Values (
            Values::Some(v.iter().map(|s| Value::from(*s)).collect())
        )
    }
}

impl From<&str> for HasObject {
    fn from(v: &str) -> Self {
        HasObject::Values (
            Values::Some(vec![v.into()])
        )
    }
}

impl From<String> for HasObject {
    fn from(v: String) -> Self {
        HasObject::Values (
            Values::Some(vec![v.into()])
        )
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


#[derive(Clone)]
pub enum Tag {
    None,
    Some(String),
}

impl From<Option<String>> for Tag {
    fn from(v: Option<String>) -> Self {
        match v {
            Some(v) => Tag::Some(v),
            None => Tag::None
        }
    }
}

impl From<String> for Tag {
    fn from(v: String) -> Self {
        Tag::Some(v)
    }
}

impl From<&str> for Tag {
    fn from(v: &str) -> Self {
        Tag::Some(v.into())
    }
}




#[derive(Clone)]
pub enum SaveVia {
    Value(Value),
    Path(path::Path),
}

impl SaveVia {
    pub fn to_via(self) -> path::Via {
        return match self {
            SaveVia::Path(path) => path::Via::Path(path),
            SaveVia::Value(value) => path::Via::Values(vec![value])
        };
    }
}


impl From<String> for SaveVia {
    fn from(v: String) -> Self {
        SaveVia::Value(v.into())
    }
}

impl From<&str> for SaveVia {
    fn from(v: &str) -> Self {
        SaveVia::Value(v.into())
    }
}

impl From<Value> for SaveVia {
    fn from(v: Value) -> Self {
        SaveVia::Value(v)
    }
}

impl From<&Path> for SaveVia {
    fn from(p: &Path) -> SaveVia {
        SaveVia::Path(p.clone().path)
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

pub fn regex<S: Into<String>>(pattern: S, iri: bool) -> Rc<dyn shape::ValueFilter> {
    Rc::new(shape::Regexp::new(pattern.into(), iri))
}

pub fn like<S: Into<String>>(pattern: S) -> Rc<dyn shape::ValueFilter> {
    Rc::new(shape::Wildcard::new(pattern.into()))
}