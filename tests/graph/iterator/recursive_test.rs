use gizmo_graph_db::graph::iterator::fixed::{Fixed};
use gizmo_graph_db::graph::iterator::and::{And};
use gizmo_graph_db::graph::iterator::save::{tag};
use gizmo_graph_db::graph::iterator::recursive::{Recursive};
use gizmo_graph_db::graph::iterator::{Shape, Morphism};
use gizmo_graph_db::graph::refs::{pre_fetched, Namer};
use gizmo_graph_db::graph::value::{Value};
use gizmo_graph_db::graph::linksto::{LinksTo};
use gizmo_graph_db::graph::hasa::{HasA};
use gizmo_graph_db::graph::graphmock::{Store};
use gizmo_graph_db::graph::quad::{Quad, QuadStore, Direction};
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;


struct SingleHop {
    qs: Rc<RefCell<dyn QuadStore>>,
    pred: String
}

impl Morphism for SingleHop {
    fn morph(&self, shape: Rc<RefCell<dyn Shape>>) -> Rc<RefCell<dyn Shape>> {
        let fixed = Fixed::new(vec![]);
        fixed.borrow_mut().add(pre_fetched(Value::from(self.pred.clone())));
        let pred_lto = LinksTo::new(self.qs.clone(), fixed, Direction::Predicate);
        let lto = LinksTo::new(self.qs.clone(), shape.clone(), Direction::Subject);
        let and = And::new(vec![]);
        and.borrow_mut().add_sub_iterator(lto);
        and.borrow_mut().add_sub_iterator(pred_lto);
        return HasA::new(self.qs.clone(), and, Direction::Object)
    }
}


fn rec_test_qs() -> Store {
    Store {
        data: vec![
            Quad::new("alice", "parent", "bob", ""),
            Quad::new("bob", "parent", "charlie", ""),
            Quad::new("charlie", "parent", "dani", ""),
            Quad::new("charlie", "parent", "bob", ""),
            Quad::new("dani", "parent", "emily", ""),
            Quad::new("fred", "follows", "alice", ""),
            Quad::new("greg", "follows", "alice", ""),
        ].into_iter().collect()
    }
}



#[test]
fn test_recursive_next() {
    let qs = Rc::new(RefCell::new(rec_test_qs()));
    let start = Fixed::new(vec![]);
    start.borrow_mut().add(pre_fetched(Value::from("alice")));
    let r = Recursive::new(start, Rc::new(SingleHop {qs: qs.clone(), pred: "parent".to_string()}), 0).borrow().iterate();

    let mut expected = vec!["bob", "charlie", "dani", "emily"];
    let mut got = Vec::new();
    while r.borrow_mut().next() {
        got.push(r.borrow().result().unwrap().key().unwrap().to_string());
    }

    expected.sort();
    got.sort();

    assert_eq!(expected, got);
}


#[test]
fn test_recursive_contains() {
    let qs = Rc::new(RefCell::new(rec_test_qs()));
    let start = Fixed::new(vec![]);
    start.borrow_mut().add(pre_fetched(Value::from("alice")));
    let r = Recursive::new(start, Rc::new(SingleHop {qs: qs.clone(), pred: "parent".to_string()}), 0).borrow().lookup();
    let values = vec!["charlie", "bob", "not"];
    let expected = vec![true, true, false];

    for i in 0..values.len() {
        let v = values[i];

        let value = qs.borrow().value_of(&Value::from(v));
        let ok = value.is_some() && r.borrow_mut().contains(value.as_ref().unwrap()); 
        
        assert_eq!(expected[i], ok);
    }
}


#[test]
fn test_recursive_next_path() {
    let qs = Rc::new(RefCell::new(rec_test_qs()));

    let start = qs.borrow().nodes_all_iterator();
    let start = tag(&start, &"person");
    
    let it = SingleHop {qs: qs.clone(), pred: "follows".to_string()}.morph(start);

    let and = And::new(vec![]);
    and.borrow_mut().add_sub_iterator(it);

    let fixed = Fixed::new(vec![]);
    fixed.borrow_mut().add(pre_fetched(Value::from("alice")));

    and.borrow_mut().add_sub_iterator(fixed);

    let r = Recursive::new(and, Rc::new(SingleHop {qs: qs.clone(), pred: "parent".to_string()}), 0).borrow().iterate();

    let mut expected = vec!["fred", "fred", "fred", "fred", "greg", "greg", "greg", "greg"];
    let mut got = Vec::new();

    while r.borrow_mut().next() {
        let mut res = HashMap::new();
        r.borrow().tag_results(&mut res);
        got.push(res[&"person".to_string()].key().unwrap().to_string());
        while r.borrow_mut().next_path() {
            let mut res = HashMap::new();
            r.borrow().tag_results(&mut res);
            got.push(res[&"person".to_string()].key().unwrap().to_string());
        }
    }

    expected.sort();
    got.sort();

    assert_eq!(expected, got);
}