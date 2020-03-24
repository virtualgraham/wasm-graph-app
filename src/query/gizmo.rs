use super::path;
use super::shape;
use std::rc::Rc;
use std::cell::RefCell;
use crate::graph::quad::{QuadStore, QuadWriter, IgnoreOptions, Quad};
use crate::graph::graphmock;
use crate::graph::value::{Value, Values};
use crate::graph::iterator;
use io_context::Context;
use std::collections::HashMap;
use crate::graph::refs::Ref;

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

impl Session {
    pub fn write(&self, quads: Vec<Quad>) {
        for quad in &quads {
            self.qw.add_quad(quad.clone()).unwrap();
        }
    }

    pub fn read(&self) -> Vec<Quad> {
        // TODO: implement
        vec![Quad::new("a", "b", "c", "d")]
    }

    pub fn delete(&self, quads: Vec<Quad>) {
        // TODO: implement
    }

    fn run_tag_each_iterator(&mut self, it: Rc<RefCell<dyn iterator::Shape>>) -> iterator::iterate::TagEachIterator {
        iterator::iterate::TagEachIterator::new(self.ctx.clone(), it, false, self.limit, true)
    }

    fn run_each_iterator(&mut self, it: Rc<RefCell<dyn iterator::Shape>>) -> iterator::iterate::EachIterator {
        iterator::iterate::EachIterator::new(self.ctx.clone(), it, false, self.limit, true)
    }
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

    fn build_iterator_tree(&self) -> Rc<RefCell<dyn iterator::Shape>> {
        let s = self.session.borrow();
        let ctx = s.ctx.borrow();

        let qs = self.session.borrow().qs.clone();

        self.path.build_iterator_on(&*ctx, qs)
    }


    ///////////////
    // Finals
    ///////////////

    pub fn get_limit(&self, limit: i64) -> impl Iterator<Item = HashMap<String, Value>> {
        let it = self.build_iterator_tree();
        let it = iterator::save::tag(&it, &"id");
        self.session.borrow_mut().limit = limit; 
        let qs = self.session.borrow().qs.clone();
        self.session.borrow_mut().run_tag_each_iterator(it).filter_map(move |r| tags_to_value_map(&r, &*qs.borrow()))
    }

    pub fn all(&mut self) -> impl Iterator<Item = HashMap<String, Value>> {
        let limit = self.session.borrow().limit;
        self.get_limit(limit)
    }

    pub fn get_limit_values(&self, limit: i64) -> impl Iterator<Item = Value> {
        let it = self.build_iterator_tree();
        self.session.borrow_mut().limit = limit; 
        let qs = self.session.borrow().qs.clone();
        self.session.borrow_mut().run_each_iterator(it).filter_map(move |r| ref_to_value(&r, &*qs.borrow()))
    }

    pub fn all_values(&mut self) -> impl Iterator<Item = Value> {
        let limit = self.session.borrow().limit;
        self.get_limit_values(limit)
    }


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

    ///////////////////////////
    // In(values: String[], tags: String[])
    ///////////////////////////
    pub fn r#in<V: Into<path::Via>>(&mut self, via: V, tags: Option<Vec<String>>) -> &mut Path {
        let tags:Vec<String> = if let Some(t) = tags { t } else { Vec::new() };
        self.path.in_with_tags(tags, via.into());
        self
    }

    ///////////////////////////
    // Out(values: String[], tags: String[])
    ///////////////////////////
    pub fn out<V: Into<path::Via>>(&mut self, via: V, tags: Option<Vec<String>>) -> &mut Path {
        let tags:Vec<String> = if let Some(t) = tags { t } else { Vec::new() };
        self.path.out_with_tags(tags, via.into());
        self
    }

    ///////////////////////////
    // Both(values: String[], tags: String[])
    ///////////////////////////
    pub fn both<V: Into<path::Via>>(&mut self, via: V, tags: Option<Vec<String>>) -> &mut Path {
        let tags:Vec<String> = if let Some(t) = tags { t } else { Vec::new() };
        self.path.both_with_tags(tags, via.into());
        self
    }

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
    ///////////////////////////
    
    pub fn and(&mut self, path: &Path) -> &mut Path {
        self.intersect(path)
    }


    ///////////////////////////
    // Intersect(path: Path)
    ///////////////////////////

    pub fn intersect(&mut self, path: &Path) -> &mut Path {
        self.path.and(path.path.clone());
        self
    }


    ///////////////////////////
    // Or(path: Path)
    ///////////////////////////
    
    pub fn or(&mut self, path: &Path) -> &mut Path {
        self.union(path)
    }


    /////////////////////////// 
    // Union(path: Path)
    ///////////////////////////

    pub fn union(&mut self, path: &Path) -> &mut Path {
        self.path.or(path.path.clone());
        self
    }


    ///////////////////////////
    // Back(tag: String)
    ///////////////////////////
    pub fn back<S: Into<String>>(&mut self, tag: S) -> &mut Path {
        let np = self.path.back(tag.into());
        if let Some(p) = np {
            self.path = p
        }
        self
    }


    ///////////////////////////
    // Back(tags: String[])
    ///////////////////////////
    pub fn tag(&mut self, tags: Vec<String>) -> &mut Path {
        self.path.tag(tags);
        self
    }


    ///////////////////////////
    // As(tags: String[])
    ///////////////////////////
    pub fn r#as(&mut self, tags: Vec<String>) -> &mut Path {
        self.tag(tags)
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
        self.path.except(path.path.clone());
        self
    }

    ///////////////////////////
    // Unique()
    ///////////////////////////
    pub fn unique(&mut self) -> &mut Path {
        self.path.unique();
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

pub fn regex<S: Into<String>>(pattern: S, iri: bool) -> Rc<dyn shape::ValueFilter> {
    Rc::new(shape::Regexp::new(pattern.into(), iri))
}

pub fn like<S: Into<String>>(pattern: S) -> Rc<dyn shape::ValueFilter> {
    Rc::new(shape::Wildcard::new(pattern.into()))
}