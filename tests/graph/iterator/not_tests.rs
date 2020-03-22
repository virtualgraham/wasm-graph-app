use io_context::Context;
use gizmo_graph_db::graph::iterator::fixed::{Fixed};
use gizmo_graph_db::graph::iterator::not::{Not};
use gizmo_graph_db::graph::iterator::{Shape};
use gizmo_graph_db::graph::refs::{Ref};
use super::common;


#[test]
fn test_not_iterator_basics() {
    let ctx = Context::background();

    let all_it = Fixed::new(vec![
        Ref::new_i64_node(1),
        Ref::new_i64_node(2),
        Ref::new_i64_node(3),
        Ref::new_i64_node(4),
    ]);
    
    let to_compliment_it = Fixed::new(vec![
        Ref::new_i64_node(2),
        Ref::new_i64_node(4),
    ]);

    let not = Not::new(to_compliment_it, all_it);

    let st = not.borrow_mut().stats(&ctx);
    assert_eq!(2, st.unwrap().size.value);

    let expect = vec![1,3];
    for _ in 0..2 {
        assert_eq!(expect, common::iterated(not.clone()));
    }

    let nc = not.borrow().lookup();
    for v in vec![1,3] {
        assert!(nc.borrow_mut().contains(&ctx, &Ref::new_i64_node(v)));
    }

    for v in vec![2,4] {
        assert!(!nc.borrow_mut().contains(&ctx, &Ref::new_i64_node(v)));
    }
}


#[test]
fn test_not_iterator_err() {
    let ctx = Context::background();
    let all_it = common::Test::new(false, Some("unique".to_string()));
    
    let to_complement_it = Fixed::new(vec![]);

    let not = Not::new(to_complement_it, all_it).borrow().iterate();

    assert!(!not.borrow_mut().next(&ctx));
    assert_eq!(Some("unique".to_string()), not.borrow().err());
}