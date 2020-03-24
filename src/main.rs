mod query;
mod graph;

use query::gizmo;
use graph::quad::Quad;

#[macro_use]
extern crate serde_derive;

#[macro_use] 
extern crate maplit;

use graph::value::Value;
use std::collections::HashMap;


fn main() {
 

    // {

    //     let multi_graph = gizmo::new_memory_graph();
    //     let session = multi_graph.s.borrow_mut();

    //     session.write(vec![Quad::new("<alice>", "<follows>", "<bob>", "")]);
    //     session.write(vec![Quad::new("<bob>", "<follows>", "<fred>", "")]);
    //     session.write(vec![Quad::new("<bob>", "<status>", "cool_person", "")]);

    //     session.write(vec![Quad::new("<dani>", "<follows>", "<bob>", "")]);
    //     session.write(vec![Quad::new("<charlie>", "<follows>", "<dani>", "")]);
    //     session.write(vec![Quad::new("<dani>", "<follows>", "<bob>", "")]);

    //     session.write(vec![Quad::new("<dani>", "<follows>", "<greg>", "")]);
    //     session.write(vec![Quad::new("<dani>", "<status>", "cool_person", "")]);
    //     session.write(vec![Quad::new("<emily>", "<follows>", "<fred>", "")]);

    //     session.write(vec![Quad::new("<fred>", "<follows>", "<greg>", "")]);
    //     session.write(vec![Quad::new("<greg>", "<status>", "cool_person", "")]);
    //     session.write(vec![Quad::new("<predicates>", "<are>", "<follows>", "")]);

    //     session.write(vec![Quad::new("<predicates>", "<are>", "<status>", "")]);
    //     session.write(vec![Quad::new("<emily>", "<status>", "smart_person", "<smart_graph>")]);
    //     session.write(vec![Quad::new("<greg>", "<status>", "smart_person", "<smart_graph>")]);

    //     session.write(vec![Quad::new("<fred>", "<status>", "smart_person", "<smart_graph>")]);

    // }
   
    
    let mut simple_graph = gizmo::new_memory_graph();

    {
        let session = simple_graph.s.borrow_mut();

        session.write(vec![Quad::new("<alice>", "<follows>", "<bob>", ())]);
        session.write(vec![Quad::new("<bob>", "<follows>", "<fred>", ())]);
        session.write(vec![Quad::new("<bob>", "<status>", "cool_person", ())]);

        session.write(vec![Quad::new("<dani>", "<follows>", "<bob>", ())]);
        session.write(vec![Quad::new("<charlie>", "<follows>", "<bob>", ())]);
        session.write(vec![Quad::new("<charlie>", "<follows>", "<dani>", ())]);

        session.write(vec![Quad::new("<dani>", "<follows>", "<greg>", ())]);
        session.write(vec![Quad::new("<dani>", "<status>", "cool_person", ())]);
        session.write(vec![Quad::new("<emily>", "<follows>", "<fred>", ())]);

        session.write(vec![Quad::new("<fred>", "<follows>", "<greg>", ())]);
        session.write(vec![Quad::new("<greg>", "<status>", "cool_person", ())]);
        session.write(vec![Quad::new("<predicates>", "<are>", "<follows>", ())]);

        session.write(vec![Quad::new("<predicates>", "<are>", "<status>", ())]);
        session.write(vec![Quad::new("<emily>", "<status>", "smart_person", ())]);
        session.write(vec![Quad::new("<greg>", "<status>", "smart_person", ())]);

    }

    

    let g = simple_graph.g();


    let r:Vec<HashMap<String, Value>> = g
        .v("<emily>")
        .out("<follows>", None)
        .r#as(vec!["f".into()])
        .out("<follows>", None)
        .out("<status>", None)
        .is("cool_person")
        .back("f")
        .r#in("<follows>", None)
        .r#in("<follows>", None)
        .r#as(vec!["acd".into()])
        .out("<status>", None)
        .is("cool_person")
        .back("f")
        .all().collect(); // just pred labels

 
   
    println!("{:?}", r);


}