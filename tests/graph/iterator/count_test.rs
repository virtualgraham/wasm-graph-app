use io_context::Context;
use cayley_wasm::graph::iterator::fixed::{Fixed};
use cayley_wasm::graph::iterator::count::{Count};
use cayley_wasm::graph::iterator::and::{And};
use cayley_wasm::graph::iterator::{Shape};
use cayley_wasm::graph::refs::{pre_fetched};
use cayley_wasm::graph::value::{Value};


#[test]
fn test_count() {
    let ctx = Context::background();

    let fixed = Fixed::new(vec![
        pre_fetched(Value::from("a")),
        pre_fetched(Value::from("b")),
        pre_fetched(Value::from("c")),
        pre_fetched(Value::from("d")),
        pre_fetched(Value::from("e")),
    ]);

    let its = Count::new(fixed.clone(), None);

    let itn = its.borrow().iterate();
    assert!(itn.borrow_mut().next(&ctx));
    assert_eq!(pre_fetched(Value::from(5)), itn.borrow().result().unwrap());
    assert!(!itn.borrow_mut().next(&ctx));

    let itc = its.borrow().lookup();
    assert!(itc.borrow_mut().contains(&ctx, &pre_fetched(Value::from(5))));
    assert!(!itc.borrow_mut().contains(&ctx, &pre_fetched(Value::from(3))));

    let fixed2 = Fixed::new(vec![
        pre_fetched(Value::from("b")),
        pre_fetched(Value::from("d")),
    ]);

    let its = Count::new(And::new(vec![fixed.clone(), fixed2]), None);

    let itn = its.borrow().iterate();
    assert!(itn.borrow_mut().next(&ctx));
    assert_eq!(pre_fetched(Value::from(2)), itn.borrow().result().unwrap());
    assert!(!itn.borrow_mut().next(&ctx));

    let itc = its.borrow().lookup();
    assert!(!itc.borrow_mut().contains(&ctx, &pre_fetched(Value::from(5))));
    assert!(itc.borrow_mut().contains(&ctx, &pre_fetched(Value::from(2))));
}
