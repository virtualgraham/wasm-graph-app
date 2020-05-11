use gizmo_graph_db::graph::iterator::and::{And};
use gizmo_graph_db::graph::iterator::save::{tag};
use gizmo_graph_db::graph::iterator::{Shape, Null, is_null};
use gizmo_graph_db::graph::refs::{Ref, Size};
use gizmo_graph_db::graph::iterator::fixed::{Fixed};
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;
use super::common;


#[test]
fn test_and_tag() {
    let fix1:Rc<RefCell<dyn Shape>> = Fixed::new(vec![Ref::new_i64_node(234)]);
    let fix2:Rc<RefCell<dyn Shape>> = Fixed::new(vec![Ref::new_i64_node(234)]);
    let ands = And::new(vec![tag(&fix1, &"foo")]);
    ands.borrow_mut().add_optional_iterator(tag(&fix2, &"baz"));
    let ands = tag(&(ands as Rc<RefCell<dyn Shape>>), &"bar");

    let and = ands.borrow().iterate();
    assert!(and.borrow_mut().next());
    assert_eq!(Ref::new_i64_node(234), and.borrow().result().unwrap());

    let mut tags:HashMap<String, Ref> = HashMap::new();
    and.borrow().tag_results(&mut tags);

    assert_eq!(hashmap!{
        "foo".into() => Ref::new_i64_node(234),
        "bar".into() => Ref::new_i64_node(234),
        "baz".into() => Ref::new_i64_node(234),
    }, tags)
}


#[test]
fn test_and_and_fixed_iterators() {
    let fix1:Rc<RefCell<dyn Shape>> = Fixed::new(vec![
        Ref::new_i64_node(1),
        Ref::new_i64_node(2),
        Ref::new_i64_node(3),
        Ref::new_i64_node(4),
    ]);
    let fix2:Rc<RefCell<dyn Shape>> = Fixed::new(vec![
        Ref::new_i64_node(3),
        Ref::new_i64_node(4),
        Ref::new_i64_node(5),
    ]);
    let ands = And::new(vec![fix1, fix2]);

    let st = ands.borrow_mut().stats();
    
    assert_eq!(st.unwrap().size, Size {
        value: 3, 
        exact: true
    });

    let and = ands.borrow().iterate();

    assert!(and.borrow_mut().next());
    assert_eq!(Ref::new_i64_node(3), and.borrow().result().unwrap());

    assert!(and.borrow_mut().next());
    assert_eq!(Ref::new_i64_node(4), and.borrow().result().unwrap());

    assert!(!and.borrow_mut().next());
}


#[test]
fn test_non_overlapping_fixed_iterators() {
    let fix1:Rc<RefCell<dyn Shape>> = Fixed::new(vec![
        Ref::new_i64_node(1),
        Ref::new_i64_node(2),
        Ref::new_i64_node(3),
        Ref::new_i64_node(4),
    ]);
    let fix2:Rc<RefCell<dyn Shape>> = Fixed::new(vec![
        Ref::new_i64_node(5),
        Ref::new_i64_node(6),
        Ref::new_i64_node(7),
    ]);
    let ands = And::new(vec![fix1, fix2]);

    let st = ands.borrow_mut().stats();
    assert_eq!(st.unwrap().size, Size {
        value: 3, 
        exact: true
    });

    let and = ands.borrow().iterate();
    assert!(!and.borrow_mut().next());
}


#[test]
fn test_all_iterators() {
    let all1 = common::Int64::new(1, 5, true);
    let all2 = common::Int64::new(4, 10, true);
    let and = And::new(vec![all1, all2]).borrow().iterate();

    assert!(and.borrow_mut().next());
    assert_eq!(Ref::new_i64_node(4), and.borrow().result().unwrap());

    assert!(and.borrow_mut().next());
    assert_eq!(Ref::new_i64_node(5), and.borrow().result().unwrap());

    assert!(!and.borrow_mut().next());
}


#[test]
fn test_and_iterator_err() {
    let all_err = common::Test::new(false, Some("Unique".to_string()));

    let and = And::new(vec![all_err, common::Int64::new(1,5, true)]).borrow().iterate();

    assert!(!and.borrow_mut().next());
    assert_eq!(Some("Unique".to_string()), and.borrow().err());
}


#[test]
fn test_null_iterator_and() {
    let all = common::Int64::new(1, 3, true);
    let null = Null::new();
    let a = And::new(vec![all, null]);
    let new_it = a.borrow_mut().optimize();
    if new_it.is_none() {
        panic!("Didn't Change")
    } 

    assert!( is_null( &mut*new_it.as_ref().unwrap().borrow_mut() ) );
}


#[test]
fn test_reorder_with_tag() {
    let all = Fixed::new(vec![Ref::new_i64_node(3)]);
    let all2 = Fixed::new(vec![
        Ref::new_i64_node(3),
        Ref::new_i64_node(4),
        Ref::new_i64_node(5),
        Ref::new_i64_node(6),
    ]);
    let a = And::new(vec![]);
    a.borrow_mut().add_sub_iterator(all2);
    a.borrow_mut().add_sub_iterator(all);

    let result = a.borrow_mut().optimize();
    assert!(result.is_some())
}


#[test]
fn test_and_statistics() {
    let all = common::Int64::new(100, 300, true);
    let all2 = common::Int64::new(1, 30000, true);
    let a = And::new(vec![]);

    a.borrow_mut().add_sub_iterator(all2);
    a.borrow_mut().add_sub_iterator(all);

    let stats1 = a.borrow_mut().stats();
    let new_it = a.borrow_mut().optimize();
    assert!(new_it.is_some());

    let stats2 = new_it.unwrap().borrow_mut().stats();
    assert!(stats2.unwrap().next_cost <= stats1.unwrap().next_cost);
}