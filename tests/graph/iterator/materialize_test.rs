use io_context::Context;
use cayley_wasm::graph::iterator::materialize::{Materialize, MATERIALIZE_LIMIT};
use cayley_wasm::graph::iterator::or::{Or};
use cayley_wasm::graph::iterator::{Shape};
use super::common;


#[test]
fn test_materialize_iterator_error() {
    let ctx = Context::background();

    let err_it = common::Test::new(false, Some("unique".to_string()));

    let m_it = Materialize::new(err_it).borrow().iterate();

    assert!(!m_it.borrow_mut().next(&ctx));
    assert_eq!(Some("unique".to_string()), m_it.borrow().err());
}


#[test]
fn test_materialize_iterator_error_abort() {
    let ctx = Context::background();
    let err_it = common::Test::new(false, Some("unique".to_string()));

    let or = Or::new(vec![common::Int64::new(1, (MATERIALIZE_LIMIT+1) as i64, true), err_it]);

    let m_it = Materialize::new(or).borrow().iterate();

    for _ in 0..(MATERIALIZE_LIMIT+1) {
        assert!(m_it.borrow_mut().next(&ctx));
        assert!(m_it.borrow().err().is_none());
    }

    assert!(!m_it.borrow_mut().next(&ctx));
    assert_eq!(Some("unique".to_string()), m_it.borrow().err());
}