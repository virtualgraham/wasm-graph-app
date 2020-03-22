use io_context::Context;

use gizmo_graph_db::graph::iterator::resolver::{Resolver};
use gizmo_graph_db::graph::iterator::{Shape};
use gizmo_graph_db::graph::refs::{pre_fetched, Namer};
use gizmo_graph_db::graph::value::{Value};
use gizmo_graph_db::graph::graphmock::{Store};
use gizmo_graph_db::graph::quad::{Quad};
use std::rc::Rc;
use std::collections::HashMap;




#[test]
fn test_resolver_iterator_iterate() {
    let ctx = Context::background();

    let nodes = vec![
        Value::from("1"),
        Value::from("2"),
        Value::from("3"),
        Value::from("4"),
        Value::from("5"),
        Value::from("3"),
    ];

    let data = nodes.iter().map(|n| {
        Quad::new(Value::from("0"), Value::from("has"), n.clone(), Value::from(""))
    }).collect();

    let qs = Rc::new(Store {
        data
    });
    let mut expected = HashMap::new();
    for node in &nodes {
        expected.insert(node, qs.value_of(node));
    }
    let it = Resolver::new(qs.clone(), nodes.clone()).borrow().iterate();
    for node in &nodes {
        assert!(it.borrow_mut().next(&ctx));
        assert!(it.borrow().err().is_none());
        assert_eq!(expected[node], it.borrow().result());
    }
    assert!(!it.borrow_mut().next(&ctx));
    assert!(it.borrow_mut().result().is_none());
}

#[test]
fn test_resolver_iterator_not_found_error() {
    let ctx = Context::background();

    let nodes = vec![
        Value::from("1"),
        Value::from("2"),
        Value::from("3"),
        Value::from("4"),
        Value::from("5")
    ];

    let data = nodes.iter().filter(|n| n != &&Value::from("3")).map(|n| {
        Quad::new(Value::from("0"), Value::from("has"), n.clone(), Value::from(""))
    }).collect();

    let qs = Rc::new(Store {
        data
    });

    let mut count = 0;
    let it = Resolver::new(qs, nodes).borrow().iterate();
    while it.borrow_mut().next(&ctx) { 
        count += 1; 
    }
    assert_eq!(0, count);
    assert!(it.borrow().err().is_some());
    assert!(it.borrow().result().is_none());
}


#[test]
fn test_resolver_iterator_contains() {
    let ctx = Context::background();

    let test = |nodes: Vec<Value>, subject:Value, contains:bool| {
        let data = nodes.iter().map(|n| {
            Quad::new(Value::from("0"), Value::from("has"), n.clone(), Value::from(""))
        }).collect();

        let qs = Rc::new(Store {
            data
        });

        let it = Resolver::new(qs, nodes).borrow().lookup();
        assert_eq!(contains, it.borrow_mut().contains(&ctx, &pre_fetched(subject)));
    };

    test(vec![
        Value::from("1"),
        Value::from("2"),
        Value::from("3")
    ], Value::from("2"), true);

    test(vec![
        Value::from("1"),
        Value::from("3")
    ], Value::from("2"), false);
}