use io_context::Context;
use gizmo_graph_db::graph::iterator::fixed::{Fixed};
use gizmo_graph_db::graph::iterator::{Shape};
use gizmo_graph_db::graph::refs::{Namer};
use gizmo_graph_db::graph::value::{Value};
use gizmo_graph_db::graph::quad::{Quad, QuadStore, Direction};
use gizmo_graph_db::graph::linksto::{LinksTo};
use gizmo_graph_db::graph::graphmock::{Store};
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashSet;

#[test]
fn test_links_to() {
    let ctx = Context::background();
    let object = Value::from("cool");
    let q = Quad {
        subject: Value::from("alice"),
        predicate: Value::from("is"),
        object: object.clone(),
        label: Value::from(""),
    };
    let qs = Rc::new(RefCell::new(Store {
        data: vec![q.clone()].into_iter().collect()
    }));
    let fixed = Fixed::new(vec![]);

    let val = qs.borrow().value_of(&object).unwrap();

    fixed.borrow_mut().add(val);
    let lto = LinksTo::new(qs.clone(), fixed, Direction::Object).borrow().iterate();

    assert!(lto.borrow_mut().next(&ctx));
    assert_eq!(q, qs.borrow().quad(lto.borrow().result().as_ref().unwrap()).unwrap());
}