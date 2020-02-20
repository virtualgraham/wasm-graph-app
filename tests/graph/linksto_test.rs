use io_context::Context;
use cayley_wasm::graph::iterator::fixed::{Fixed};
use cayley_wasm::graph::iterator::{Shape};
use cayley_wasm::graph::refs::{Namer};
use cayley_wasm::graph::value::{Value};
use cayley_wasm::graph::quad::{Quad, QuadIndexer, Direction};
use cayley_wasm::graph::linksto::{LinksTo};
use cayley_wasm::graph::graphmock::{Store};
use std::rc::Rc;



#[test]
fn test_links_to() {
    let ctx = Context::background();
    let object = Value::from("cool");
    let q = Quad {
        subject: Value::from("alice"),
        predicate: Value::from("is"),
        object: object.clone(),
        label: Value::from(""),
    };
    let qs = Rc::new(Store {
        data: vec![q.clone()]
    });
    let fixed = Fixed::new(vec![]);

    let val = qs.value_of(&object).unwrap();

    fixed.borrow_mut().add(val);
    let lto = LinksTo::new(qs.clone(), fixed, Direction::Object).borrow().iterate();

    assert!(lto.borrow_mut().next(&ctx));
    assert_eq!(q, qs.quad(lto.borrow().result().as_ref().unwrap()));
}