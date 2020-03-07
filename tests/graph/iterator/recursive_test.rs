use io_context::Context;
use cayley_wasm::graph::iterator::fixed::{Fixed};
use cayley_wasm::graph::iterator::and::{And};
use cayley_wasm::graph::iterator::save::{tag};
use cayley_wasm::graph::iterator::recursive::{Recursive};
use cayley_wasm::graph::iterator::{Shape, Morphism};
use cayley_wasm::graph::refs::{pre_fetched, Namer};
use cayley_wasm::graph::value::{Value};
use cayley_wasm::graph::linksto::{LinksTo};
use cayley_wasm::graph::hasa::{HasA};
use cayley_wasm::graph::graphmock::{Store};
use cayley_wasm::graph::quad::{Quad, QuadIndexer, QuadStore, Direction};
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;


struct SingleHop {
    qs: Rc<dyn QuadIndexer>,
    pred: String
}

impl Morphism for SingleHop {
    fn morph(&self, it: &Rc<RefCell<dyn Shape>>) -> Rc<RefCell<dyn Shape>> {
        let fixed = Fixed::new(vec![]);
        fixed.borrow_mut().add(pre_fetched(Value::from(self.pred.clone())));
        let pred_lto = LinksTo::new(self.qs.clone(), fixed, Direction::Predicate);
        let lto = LinksTo::new(self.qs.clone(), it.clone(), Direction::Subject);
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
        ]
    }
}



#[test]
fn test_recursive_next() {
    let ctx = Context::background();
    let qs = Rc::new(rec_test_qs());
    let start = Fixed::new(vec![]);
    start.borrow_mut().add(pre_fetched(Value::from("alice")));
    let r = Recursive::new(start, Rc::new(SingleHop {qs: qs.clone(), pred: "parent".to_string()}), 0).borrow().iterate();

    let mut expected = vec!["bob", "charlie", "dani", "emily"];
    let mut got = Vec::new();
    while r.borrow_mut().next(&ctx) {
        got.push(r.borrow().result().unwrap().key.to_string());
    }

    expected.sort();
    got.sort();

    assert_eq!(expected, got);
}


#[test]
fn test_recursive_contains() {
    let ctx = Context::background();
    let qs = Rc::new(rec_test_qs());
    let start = Fixed::new(vec![]);
    start.borrow_mut().add(pre_fetched(Value::from("alice")));
    let r = Recursive::new(start, Rc::new(SingleHop {qs: qs.clone(), pred: "parent".to_string()}), 0).borrow().lookup();
    let values = vec!["charlie", "bob", "not"];
    let expected = vec![true, true, false];

    for i in 0..values.len() {
        let v = values[i];

        let value = qs.value_of(&Value::from(v));
        let ok = value.is_some() && r.borrow_mut().contains(&ctx, value.as_ref().unwrap()); 
        
        assert_eq!(expected[i], ok);
    }
}


#[test]
fn test_recursive_next_path() {
    let ctx = Context::background();
    let qs = Rc::new(rec_test_qs());

    let start = qs.nodes_all_iterator();
    let start = tag(&start, &"person");
    
    let it = SingleHop {qs: qs.clone(), pred: "follows".to_string()}.morph(&start);

    let and = And::new(vec![]);
    and.borrow_mut().add_sub_iterator(it);

    let fixed = Fixed::new(vec![]);
    fixed.borrow_mut().add(pre_fetched(Value::from("alice")));

    and.borrow_mut().add_sub_iterator(fixed);

    let r = Recursive::new(and, Rc::new(SingleHop {qs: qs.clone(), pred: "parent".to_string()}), 0).borrow().iterate();

    let mut expected = vec!["fred", "fred", "fred", "fred", "greg", "greg", "greg", "greg"];
    let mut got = Vec::new();

    while r.borrow_mut().next(&ctx) {
        let mut res = HashMap::new();
        r.borrow().tag_results(&mut res);
        got.push(res[&"person".to_string()].key.to_string());
        while r.borrow_mut().next_path(&ctx) {
            let mut res = HashMap::new();
            r.borrow().tag_results(&mut res);
            got.push(res[&"person".to_string()].key.to_string());
        }
    }

    expected.sort();
    got.sort();

    assert_eq!(expected, got);
}