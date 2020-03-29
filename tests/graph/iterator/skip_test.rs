use io_context::Context;
use gizmo_graph_db::graph::iterator::fixed::Fixed;
use gizmo_graph_db::graph::iterator::skip::Skip;
use gizmo_graph_db::graph::iterator::Shape;
use gizmo_graph_db::graph::refs::Ref;
use super::common;


#[test]
fn test_not_iterator_basics() {
    let ctx = Context::background();

    let all_it = Fixed::new(vec![
        Ref::new_i64_node(1),
        Ref::new_i64_node(2),
        Ref::new_i64_node(3),
        Ref::new_i64_node(4),
        Ref::new_i64_node(5),
    ]);
    
    let u = Skip::new(all_it.clone(), 0);

    let expect_sz = all_it.borrow_mut().stats(&ctx).unwrap();
    let sz = u.borrow_mut().stats(&ctx).unwrap();

    assert_eq!(expect_sz.size.value, sz.size.value);
    assert_eq!(vec![1,2,3,4,5], common::iterated(u.clone()));

    let u = Skip::new(all_it.clone(), 3);
    let sz = u.borrow_mut().stats(&ctx).unwrap();
    assert_eq!(2, sz.size.value);

    assert_eq!(vec![4,5], common::iterated(u.clone()));

    let uc = u.borrow().lookup();
    for v in &[1,2,3] {
        assert!(!uc.borrow_mut().contains(&ctx, &Ref::new_i64_node(*v as i64)))
    }
    for v in &[4,5] {
        assert!(uc.borrow_mut().contains(&ctx, &Ref::new_i64_node(*v as i64)))
    }

    let uc = u.borrow().lookup();
    for v in &[5,4,3] {
        assert!(!uc.borrow_mut().contains(&ctx, &Ref::new_i64_node(*v as i64)))
    }
    for v in &[1,2] {
        assert!(uc.borrow_mut().contains(&ctx, &Ref::new_i64_node(*v as i64)))
    }
}