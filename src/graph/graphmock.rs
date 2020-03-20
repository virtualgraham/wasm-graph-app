use super::quad::{Stats, Quad, QuadStore, Direction, Delta, IgnoreOptions, Procedure, QuadWriter};
use super::iterator::{Shape};
use super::iterator::fixed::{Fixed};
use super::value::{Value};
use super::refs::{Size, Ref, Content, pre_fetched, Namer};
use io_context::Context;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashSet;

pub struct Store {
    pub data: HashSet<Quad>
}

impl Store {
    pub fn new() -> Store {
        Store {
            data: HashSet::new()
        }
    }
}

fn quad_value(q: Quad) -> Ref {
    Ref {
        key: Value::from(q.to_string()),
        content: Content::Quad(q)
    }
}


impl Namer for Store {
    fn value_of(&self, v: &Value) -> Option<Ref> {
        for q in &self.data {
            if &q.subject == v || &q.object == v {
                return Some(pre_fetched(v.clone()))
            }
        } 
        return None
    }

    fn name_of(&self, key: &Ref) -> Option<Value> {
        return Some(key.key.clone())
    }

    #[allow(unused)]
    fn values_of(&self, ctx: &Context, values: &Vec<Ref>) -> Result<Vec<Value>, String> {
        Ok(values.iter().map(|v| self.name_of(v).unwrap()).collect())
    }

    #[allow(unused)]
    fn refs_of(&self, ctx: &Context, nodes: &Vec<Value>) -> Result<Vec<Ref>, String> {
        nodes.iter().map(|v| {
            match self.value_of(v) { Some(s) => Ok(s), None => Err("Not Found".to_string()) }
        }).collect()
    }
}

impl QuadStore for Store {
    fn quad(&self, r: &Ref) -> Quad {
        match &r.content {
            Content::Quad(q) => q.clone(),
            _ => panic!("Ref does not contain a value")
        }
    }

    fn quad_iterator(&self, d: &Direction, r: &Ref) -> Rc<RefCell<dyn Shape>> {
        let fixed = Fixed::new(vec![]);
        for q in &self.data {
            println!("Quad Iterator {:?} == {:?}, Direction: {:?}", q.get(d), &r.key, d);
            if q.get(d) == &r.key {
                fixed.borrow_mut().add(quad_value(q.clone()));
            }
        }
        return fixed
    }

    #[allow(unused)]
    fn quad_iterator_size(&self, ctx: &Context, d: &Direction, r: &Ref) -> Result<Size, String> {
        let mut sz = Size {
            value: 0,
            exact: true
        };
        for q in &self.data {
            if q.get(d) == &r.key {
                sz.value += 1;
            }
        }
        return Ok(sz);
    }

    fn quad_direction(&self, r: &Ref, d: &Direction) -> Ref {
        pre_fetched(self.quad(r).get(d).clone())
    }

    #[allow(unused)]
    fn stats(&self, ctx: &Context, exact: bool) -> Result<Stats, String> {
        let mut set = HashSet::new();
        for q in &self.data {
            for d in vec![Direction::Label, Direction::Object, Direction::Predicate, Direction::Subject] {
                let n = q.get(&d);
                set.insert(n);
            }
        }
        return Ok(Stats {
            nodes: Size {
                value: set.len() as i64,
                exact: true
            },
            quads: Size {
                value: self.data.len() as i64,
                exact: true
            }
        })
    }
    
    fn apply_deltas(&mut self, deltas: Vec<Delta>, ignore_opts: &IgnoreOptions) -> Result<(), String> {
        // if !ignore_opts.ignore_dup || !ignore_opts.ignore_missing {

        // }

        for d in deltas {
            match d.action {
                Procedure::Add => self.data.insert(d.quad),
                Procedure::Delete =>  self.data.remove(&d.quad),
            };
        }

        Ok(())
    }

    fn nodes_all_iterator(&self) -> Rc<RefCell<dyn Shape>> {
        let mut set = HashSet::new();
        for q in &self.data {
            for d in vec![Direction::Label, Direction::Object, Direction::Predicate, Direction::Subject] {
                let n = self.name_of(&pre_fetched(q.get(&d).clone()));
                if n.is_some() {
                    set.insert(n.unwrap());
                }
            }
        }
        let fixed = Fixed::new(vec![]);
        for k in set {
            fixed.borrow_mut().add(pre_fetched(k));
        }
        return fixed
    }

    fn quads_all_iterator(&self) -> Rc<RefCell<dyn Shape>> {
        let fixed = Fixed::new(vec![]);
        for q in &self.data {
            fixed.borrow_mut().add(Ref {
                key: Value::from(q.to_string()),
                content: Content::Quad(q.clone())
            });
        }
        return fixed
    }

    fn close(&self) -> Option<String> {
        return None
    }
}