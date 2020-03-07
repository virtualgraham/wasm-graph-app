use io_context::Context;
use cayley_wasm::graph::iterator::unique::{Unique};
use cayley_wasm::graph::iterator::fixed::{Fixed};
use cayley_wasm::graph::iterator::{Shape};
use cayley_wasm::graph::refs::{Ref};
use super::common;



#[test]
fn test_unique_iterator_basics() {
    let ctx = Context::background();

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
        assert!(uc.borrow_mut().contains(&ctx, &Ref::new_i64_node(v)));
    }
}