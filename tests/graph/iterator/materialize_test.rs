use gizmo_graph_db::graph::iterator::materialize::{Materialize, MATERIALIZE_LIMIT};
use gizmo_graph_db::graph::iterator::or::{Or};
use gizmo_graph_db::graph::iterator::{Shape};
use super::common;


#[test]
fn test_materialize_iterator_error() {
    let err_it = common::Test::new(false, Some("unique".to_string()));

    let m_it = Materialize::new(err_it).borrow().iterate();

    assert!(!m_it.borrow_mut().next());
    assert_eq!(Some("unique".to_string()), m_it.borrow().err());
}


#[test]
fn test_materialize_iterator_error_abort() {
    let err_it = common::Test::new(false, Some("unique".to_string()));

    let or = Or::new(vec![common::Int64::new(1, (MATERIALIZE_LIMIT+1) as i64, true), err_it]);

    let m_it = Materialize::new(or).borrow().iterate();

    for _ in 0..(MATERIALIZE_LIMIT+1) {
        assert!(m_it.borrow_mut().next());
        assert!(m_it.borrow().err().is_none());
    }

    assert!(!m_it.borrow_mut().next());
    assert_eq!(Some("unique".to_string()), m_it.borrow().err());
}