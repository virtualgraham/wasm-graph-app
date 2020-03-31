use gizmo_graph_db::graph::iterator::fixed::{Fixed};
use gizmo_graph_db::graph::iterator::count::{Count};
use gizmo_graph_db::graph::iterator::and::{And};
use gizmo_graph_db::graph::iterator::{Shape};
use gizmo_graph_db::graph::refs::{pre_fetched};
use gizmo_graph_db::graph::value::{Value};


#[test]
fn test_count() {
    let fixed = Fixed::new(vec![
        pre_fetched(Value::from("a")),
        pre_fetched(Value::from("b")),
        pre_fetched(Value::from("c")),
        pre_fetched(Value::from("d")),
        pre_fetched(Value::from("e")),
    ]);

    let its = Count::new(fixed.clone(), None);

    let itn = its.borrow().iterate();
    assert!(itn.borrow_mut().next());
    assert_eq!(pre_fetched(Value::from(5)), itn.borrow().result().unwrap());
    assert!(!itn.borrow_mut().next());

    let itc = its.borrow().lookup();
    assert!(itc.borrow_mut().contains(&pre_fetched(Value::from(5))));
    assert!(!itc.borrow_mut().contains(&pre_fetched(Value::from(3))));

    let fixed2 = Fixed::new(vec![
        pre_fetched(Value::from("b")),
        pre_fetched(Value::from("d")),
    ]);

    let its = Count::new(And::new(vec![fixed.clone(), fixed2]), None);

    let itn = its.borrow().iterate();
    assert!(itn.borrow_mut().next());
    assert_eq!(pre_fetched(Value::from(2)), itn.borrow().result().unwrap());
    assert!(!itn.borrow_mut().next());

    let itc = its.borrow().lookup();
    assert!(!itc.borrow_mut().contains(&pre_fetched(Value::from(5))));
    assert!(itc.borrow_mut().contains(&pre_fetched(Value::from(2))));
}
