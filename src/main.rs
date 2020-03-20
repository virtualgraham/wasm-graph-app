mod query;
mod graph;

use query::gizmo;
use graph::quad::Quad;

#[macro_use]
extern crate serde_derive;


fn main() {
    let simple_graph = gizmo::new_memory_graph();

    simple_graph.write(vec![Quad::new("<alice>", "<follows>", "<bob>", "")]);
    simple_graph.write(vec![Quad::new("<bob>", "<follows>", "<fred>", "")]);
    simple_graph.write(vec![Quad::new("<bob>", "<status>", "cool_person", "")]);

    simple_graph.write(vec![Quad::new("<dani>", "<follows>", "<bob>", "")]);
    simple_graph.write(vec![Quad::new("<charlie>", "<follows>", "<bob>", "")]);
    simple_graph.write(vec![Quad::new("<charlie>", "<follows>", "<dani>", "")]);

    simple_graph.write(vec![Quad::new("<dani>", "<follows>", "<greg>", "")]);
    simple_graph.write(vec![Quad::new("<dani>", "<status>", "cool_person", "")]);
    simple_graph.write(vec![Quad::new("<emily>", "<follows>", "<fred>", "")]);

    simple_graph.write(vec![Quad::new("<fred>", "<follows>", "<greg>", "")]);
    simple_graph.write(vec![Quad::new("<greg>", "<status>", "cool_person", "")]);
    simple_graph.write(vec![Quad::new("<predicates>", "<are>", "<follows>", "")]);

    simple_graph.write(vec![Quad::new("<predicates>", "<are>", "<status>", "")]);
    simple_graph.write(vec![Quad::new("<emily>", "<status>", "smart_person", "<smart_graph>")]);
    simple_graph.write(vec![Quad::new("<greg>", "<status>", "smart_person", "<smart_graph>")]);

    let multi_graph = gizmo::new_memory_graph();
    
    multi_graph.write(vec![Quad::new("<alice>", "<follows>", "<bob>", "")]);
    multi_graph.write(vec![Quad::new("<bob>", "<follows>", "<fred>", "")]);
    multi_graph.write(vec![Quad::new("<bob>", "<status>", "cool_person", "")]);

    multi_graph.write(vec![Quad::new("<dani>", "<follows>", "<bob>", "")]);
    multi_graph.write(vec![Quad::new("<charlie>", "<follows>", "<dani>", "")]);
    multi_graph.write(vec![Quad::new("<dani>", "<follows>", "<bob>", "")]);

    multi_graph.write(vec![Quad::new("<dani>", "<follows>", "<greg>", "")]);
    multi_graph.write(vec![Quad::new("<dani>", "<status>", "cool_person", "")]);
    multi_graph.write(vec![Quad::new("<emily>", "<follows>", "<fred>", "")]);

    multi_graph.write(vec![Quad::new("<fred>", "<follows>", "<greg>", "")]);
    multi_graph.write(vec![Quad::new("<greg>", "<status>", "cool_person", "")]);
    multi_graph.write(vec![Quad::new("<predicates>", "<are>", "<follows>", "")]);

    multi_graph.write(vec![Quad::new("<predicates>", "<are>", "<status>", "")]);
    multi_graph.write(vec![Quad::new("<emily>", "<status>", "smart_person", "<smart_graph>")]);
    multi_graph.write(vec![Quad::new("<greg>", "<status>", "smart_person", "<smart_graph>")]);

    multi_graph.write(vec![Quad::new("<fred>", "<status>", "smart_person", "<smart_graph>")]);

    /////
    

    let g = simple_graph.g();

    g.v(Some(vec!["<alice>".into()])).all();
}