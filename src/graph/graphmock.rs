use super::quad::{Stats, Quad, QuadStore, Direction, Delta, IgnoreOptions, Procedure, QuadWriter};
use super::iterator::{Shape};
use super::iterator::fixed::{Fixed};
use super::value::{Value};
use super::refs::{Size, Ref, Content, pre_fetched, Namer};
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashSet;

pub struct Store {
    pub data: Vec<Quad>
}

impl Store {
    pub fn new() -> Store {
        Store {
            data: Vec::new()
        }
    }
}

fn quad_value(q: Quad) -> Ref {
    Ref {
        k: Value::String(q.to_string()),
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
        return if let Some(k) = key.key() {
            Some(k.clone())
        } else {
            None
        }
    }


}

impl QuadStore for Store {
    fn quad(&self, r: &Ref) -> Option<Quad> {
        match &r.content {
            Content::Quad(q) => Some(q.clone()),
            _ => None
        }
    }


    // all quads that have the given value in the given direction
    fn quad_iterator(&self, d: &Direction, r: &Ref) -> Rc<RefCell<dyn Shape>> {
        let fixed = Fixed::new(vec![]);
        for q in &self.data {
            println!("Quad Iterator {:?} == {:?}, Direction: {:?}", q.get(d), r.key(), d);

            if let Some(k) = r.key() {
                if q.get(d) == k {
                    fixed.borrow_mut().add(quad_value(q.clone()));
                }
            }
        }
        return fixed
    }


    #[allow(unused)]
    fn quad_iterator_size(&self, d: &Direction, r: &Ref) -> Result<Size, String> {
        let mut sz = Size {
            value: 0,
            exact: true
        };
        for q in &self.data {
            if let Some(k) = r.key() {
                if q.get(d) == k {
                    sz.value += 1;
                }
            }
        }
        return Ok(sz);
    }


    fn quad_direction(&self, r: &Ref, d: &Direction) -> Option<Ref> {
        match self.quad(r) {
            Some(q) => {
                Some(pre_fetched(q.get(d).clone()))
            },
            None => None
        }
    }


    #[allow(unused)]
    fn stats(&self, exact: bool) -> Result<Stats, String> {
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
                Procedure::Add => { 
                    println!("adding quad {:?}", d.quad);
                    self.data.push(d.quad); 
                },
                Procedure::Delete =>  { 
                    let index = self.data.iter().position(|x| *x == d.quad).unwrap();
                    self.data.remove(index);
                },
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
                k: Value::String(q.to_string()),
                content: Content::Quad(q.clone())
            });
        }
        return fixed
    }


    fn close(&self) -> Option<String> {
        return None
    }
}