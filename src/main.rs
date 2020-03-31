mod query;
mod graph;

use query::gizmo;
use graph::quad::Quad;

#[macro_use]
extern crate serde_derive;


use graph::value::Value;
use std::collections::HashMap;


fn main() {
 
    let simple_graph = gizmo::new_memory_graph();

    simple_graph.write(vec![Quad::new("<alice>", "<follows>", "<bob>", ())]);
    simple_graph.write(vec![Quad::new("<bob>", "<follows>", "<fred>", ())]);
    simple_graph.write(vec![Quad::new("<bob>", "<status>", "cool_person", ())]);

    simple_graph.write(vec![Quad::new("<dani>", "<follows>", "<bob>", ())]);
    simple_graph.write(vec![Quad::new("<charlie>", "<follows>", "<bob>", ())]);
    simple_graph.write(vec![Quad::new("<charlie>", "<follows>", "<dani>", ())]);

    simple_graph.write(vec![Quad::new("<dani>", "<follows>", "<greg>", ())]);
    simple_graph.write(vec![Quad::new("<dani>", "<status>", "cool_person", ())]);
    simple_graph.write(vec![Quad::new("<emily>", "<follows>", "<fred>", ())]);

    simple_graph.write(vec![Quad::new("<fred>", "<follows>", "<greg>", ())]);
    simple_graph.write(vec![Quad::new("<greg>", "<status>", "cool_person", ())]);
    simple_graph.write(vec![Quad::new("<predicates>", "<are>", "<follows>", ())]);

    simple_graph.write(vec![Quad::new("<predicates>", "<are>", "<status>", ())]);
    simple_graph.write(vec![Quad::new("<emily>", "<status>", "smart_person", "<smart_graph>")]);
    simple_graph.write(vec![Quad::new("<greg>", "<status>", "smart_person", "<smart_graph>")]);

  

    let g = simple_graph.g();

    

    let mut r:Vec<String> = g.v("<greg>")
        .label_context("<smart_graph>", None)
        .out("<status>", None)
        .iter_values().map(|v| v.to_string()).collect();

    let mut f:Vec<String> = vec![
        "smart_person".into(),
    ];


    r.sort();
    f.sort();

    assert_eq!(r, f);


}