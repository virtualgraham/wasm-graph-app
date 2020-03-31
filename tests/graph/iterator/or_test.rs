use gizmo_graph_db::graph::iterator::fixed::{Fixed};
use gizmo_graph_db::graph::iterator::or::{Or};
use gizmo_graph_db::graph::iterator::{Shape};
use gizmo_graph_db::graph::refs::{Ref, Size};
use super::common;



#[test]
fn test_or_iterator_basics() {
    let or = Or::new(Vec::new());

    let f1 = Fixed::new(vec![
        Ref::new_i64_node(1),
        Ref::new_i64_node(2),
        Ref::new_i64_node(3),
    ]);
    
    let f2 = Fixed::new(vec![
        Ref::new_i64_node(3),
        Ref::new_i64_node(9),
        Ref::new_i64_node(20),
        Ref::new_i64_node(21),
    ]);

    or.borrow_mut().add_sub_iterator(f1);
    or.borrow_mut().add_sub_iterator(f2);

    let stats = or.borrow_mut().stats();
    

    assert_eq!(7, stats.as_ref().unwrap().size.value);

    let expect = vec![1, 2, 3, 3, 9, 20, 21];
    for i in 0..2 {
        println!("i: {}", i);
        assert_eq!(expect, common::iterated(or.clone()));
    }

    let opt_or = or.borrow_mut().optimize();
    if let Some(o) = opt_or {
        assert_eq!(expect, common::iterated(o.clone()));
    } else {
        panic!("Optimize returned None")
    }
    
    let orc = or.borrow().lookup();
    for v in vec![2,3,21] {
        assert!(orc.borrow_mut().contains(&Ref::new_i64_node(v)));
    }

    for v in vec![22, 5, 0] {
        assert!(!orc.borrow_mut().contains(&Ref::new_i64_node(v)));
    }
}


#[test]
fn test_short_circuiting_or_basics() {
    let f1 = Fixed::new(vec![
        Ref::new_i64_node(1),
        Ref::new_i64_node(2),
        Ref::new_i64_node(3),
    ]);
    
    let f2 = Fixed::new(vec![
        Ref::new_i64_node(3),
        Ref::new_i64_node(9),
        Ref::new_i64_node(20),
        Ref::new_i64_node(21),
    ]);


    let or = Or::new_short_circuit(Vec::new());
    or.borrow_mut().add_sub_iterator(f1.clone());
    or.borrow_mut().add_sub_iterator(f2.clone());

    let stats = or.borrow_mut().stats();
    assert_eq!(stats.as_ref().unwrap().size, Size {
        value: 4,
        exact: true
    });


    let or = Or::new_short_circuit(Vec::new());
    or.borrow_mut().add_sub_iterator(f1.clone());
    or.borrow_mut().add_sub_iterator(f2.clone());

    let expect = vec![1,2,3];
    for _ in 0..2 {
        assert_eq!(expect, common::iterated(or.clone()));
    }


    let opt_or = or.borrow_mut().optimize();
    if let Some(o) = opt_or {
        assert_eq!(expect, common::iterated(o.clone()));
    } else {
        panic!("Optimize returned None")
    }


    let or = Or::new_short_circuit(Vec::new());
    or.borrow_mut().add_sub_iterator(f1.clone());
    or.borrow_mut().add_sub_iterator(f2.clone());

    let orc = or.borrow().lookup();
    for v in vec![2, 3, 21] {
        assert!(orc.borrow_mut().contains(&Ref::new_i64_node(v)));
    }

    for v in vec![22, 5, 0] {
        assert!(!orc.borrow_mut().contains(&Ref::new_i64_node(v)));
    }


    let or = Or::new_short_circuit(Vec::new());
    or.borrow_mut().add_sub_iterator(Fixed::new(vec![]));
    or.borrow_mut().add_sub_iterator(f2.clone());

    let expect = vec![3, 9, 20, 21];
    for _ in 0..2 {
        assert_eq!(expect, common::iterated(or.clone()));
    }

    let opt_or = or.borrow_mut().optimize();
    if let Some(o) = opt_or {
        assert_eq!(expect, common::iterated(o.clone()));
    } else {
        panic!("Optimize returned None")
    }
}


#[test]
fn test_or_iterator_err() {
    let or_err = common::Test::new(false, Some("unique".to_string()));

    let fix1 = Fixed::new(vec![Ref::new_i64_node(1)]);

    let or = Or::new( vec![fix1, or_err, common::Int64::new(1, 5, true)] ).borrow().iterate();

    assert!(or.borrow_mut().next());
    assert_eq!(Ref::new_i64_node(1), or.borrow().result().unwrap());

    assert!(!or.borrow_mut().next());
    assert_eq!("unique", or.borrow().err().unwrap());
}


#[test]
fn test_short_circuit_or_iterator_err() {
    let or_err = common::Test::new(false, Some("unique".to_string()));

    let or = Or::new( vec![or_err, common::Int64::new(1, 5, true)] ).borrow().iterate();

    assert!(!or.borrow_mut().next());
    assert_eq!("unique", or.borrow().err().unwrap());
}