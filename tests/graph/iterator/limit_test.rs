use gizmo_graph_db::graph::iterator::fixed::{Fixed};
use gizmo_graph_db::graph::iterator::limit::{Limit};
use gizmo_graph_db::graph::iterator::{Shape};
use gizmo_graph_db::graph::refs::{Ref};
use super::common;


#[test]
fn test_limit_iterator_basics() {
    let all_it = Fixed::new(vec![
        Ref::new_i64_node(1),
        Ref::new_i64_node(2),
        Ref::new_i64_node(3),
        Ref::new_i64_node(4),
        Ref::new_i64_node(5),
    ]);

    let u = Limit::new(all_it.clone(), 0);
    let expect_sz = all_it.borrow_mut().stats();
    let sz = u.borrow_mut().stats();
    assert_eq!(expect_sz.unwrap().size.value, sz.unwrap().size.value);
    assert_eq!(vec![1,2,3,4,5], common::iterated(u));

    let u = Limit::new(all_it.clone(), 3);
    let sz = u.borrow_mut().stats();
    assert_eq!(3, sz.unwrap().size.value);
    assert_eq!(vec![1,2,3], common::iterated(u.clone()));

    let uc = u.borrow().lookup();
    for v in vec![1,2,3] {
        assert!(uc.borrow_mut().contains(&Ref::new_i64_node(v)));
    }
    assert!(!uc.borrow_mut().contains(&Ref::new_i64_node(4)));

    let uc = u.borrow().lookup();
    for v in vec![5,4,3] {
        assert!(uc.borrow_mut().contains(&Ref::new_i64_node(v)));
    }
    assert!(!uc.borrow_mut().contains(&Ref::new_i64_node(2)));
}