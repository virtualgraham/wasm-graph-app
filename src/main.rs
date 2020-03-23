mod query;
mod graph;

use query::gizmo;
use graph::quad::Quad;

#[macro_use]
extern crate serde_derive;

use graph::refs::Ref;
use std::collections::HashMap;


fn main() {
    let simple_graph = gizmo::new_memory_graph();

    {
        let session = simple_graph.s.borrow_mut();

        session.write(vec![Quad::new("<alice>", "<follows>", "<bob>", "")]);
        session.write(vec![Quad::new("<bob>", "<follows>", "<fred>", "")]);
        session.write(vec![Quad::new("<bob>", "<status>", "cool_person", "")]);

        session.write(vec![Quad::new("<dani>", "<follows>", "<bob>", "")]);
        session.write(vec![Quad::new("<charlie>", "<follows>", "<bob>", "")]);
        session.write(vec![Quad::new("<charlie>", "<follows>", "<dani>", "")]);

        session.write(vec![Quad::new("<dani>", "<follows>", "<greg>", "")]);
        session.write(vec![Quad::new("<dani>", "<status>", "cool_person", "")]);
        session.write(vec![Quad::new("<emily>", "<follows>", "<fred>", "")]);

        session.write(vec![Quad::new("<fred>", "<follows>", "<greg>", "")]);
        session.write(vec![Quad::new("<greg>", "<status>", "cool_person", "")]);
        session.write(vec![Quad::new("<predicates>", "<are>", "<follows>", "")]);

        session.write(vec![Quad::new("<predicates>", "<are>", "<status>", "")]);
        session.write(vec![Quad::new("<emily>", "<status>", "smart_person", "<smart_graph>")]);
        session.write(vec![Quad::new("<greg>", "<status>", "smart_person", "<smart_graph>")]);

        let multi_graph = gizmo::new_memory_graph();
        let session = multi_graph.s.borrow_mut();

        session.write(vec![Quad::new("<alice>", "<follows>", "<bob>", "")]);
        session.write(vec![Quad::new("<bob>", "<follows>", "<fred>", "")]);
        session.write(vec![Quad::new("<bob>", "<status>", "cool_person", "")]);

        session.write(vec![Quad::new("<dani>", "<follows>", "<bob>", "")]);
        session.write(vec![Quad::new("<charlie>", "<follows>", "<dani>", "")]);
        session.write(vec![Quad::new("<dani>", "<follows>", "<bob>", "")]);

        session.write(vec![Quad::new("<dani>", "<follows>", "<greg>", "")]);
        session.write(vec![Quad::new("<dani>", "<status>", "cool_person", "")]);
        session.write(vec![Quad::new("<emily>", "<follows>", "<fred>", "")]);

        session.write(vec![Quad::new("<fred>", "<follows>", "<greg>", "")]);
        session.write(vec![Quad::new("<greg>", "<status>", "cool_person", "")]);
        session.write(vec![Quad::new("<predicates>", "<are>", "<follows>", "")]);

        session.write(vec![Quad::new("<predicates>", "<are>", "<status>", "")]);
        session.write(vec![Quad::new("<emily>", "<status>", "smart_person", "<smart_graph>")]);
        session.write(vec![Quad::new("<greg>", "<status>", "smart_person", "<smart_graph>")]);

        session.write(vec![Quad::new("<fred>", "<status>", "smart_person", "<smart_graph>")]);

    }
    /////
    

    let g = simple_graph.g();

    // let r:Vec<HashMap<String, Ref>> = g.v("<alice>").all().collect();

    // let r:Vec<HashMap<String, Ref>> = g.v(None).all().collect();

    // let r:Vec<HashMap<String, Ref>> = g.v(None).get_limit(5).collect();

    // let r:Vec<HashMap<String, Ref>> = g.v("<alice>").out("<follows>", None).all().collect();
    
    // let r:Vec<HashMap<String, Ref>> = g.v("<bob>").out(None, None).all().collect();

    //let r:Vec<HashMap<String, Ref>> = g.v("<bob>").r#in("<follows>", None).all().collect();

    //let r:Vec<HashMap<String, Ref>> = g.v("<fred>").both("<follows>", None).all().collect();

    let r:Vec<HashMap<String, Ref>> = g.v("<bob>").r#in("<follows>", None).filter(gizmo::regex("ar?li.*e")).all().collect();

    println!("{:?} {}", r, r.len());
}