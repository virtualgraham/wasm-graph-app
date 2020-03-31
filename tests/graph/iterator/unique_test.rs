use gizmo_graph_db::graph::iterator::unique::{Unique};
use gizmo_graph_db::graph::iterator::fixed::{Fixed};
use gizmo_graph_db::graph::iterator::{Shape};
use gizmo_graph_db::graph::refs::{Ref};
use super::common;



#[test]
fn test_unique_iterator_basics() {
    let all_it = Fixed::new(vec![
        Ref::new_i64_node(1),
        Ref::new_i64_node(2),
        Ref::new_i64_node(3),
        Ref::new_i64_node(3),
        Ref::new_i64_node(2),
    ]);

    let u = Unique::new(all_it);

    let expect = vec![1,2,3];
    for _ in 0..2 {
        assert_eq!(expect, common::iterated(u.clone()));
    }

    let uc = u.borrow().lookup();
    for v in 1..4 {
        assert!(uc.borrow_mut().contains(&Ref::new_i64_node(v)));
    }
}