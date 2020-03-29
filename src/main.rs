mod query;
mod graph;

use query::gizmo;
use graph::quad::Quad;

#[macro_use]
extern crate serde_derive;


use graph::value::Value;
use std::collections::HashMap;


fn main() {
 
    let multi_graph = gizmo::new_memory_graph();
    
    {
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

    }
   
    
    let simple_graph = gizmo::new_memory_graph();

    {
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
        simple_graph.write(vec![Quad::new("<emily>", "<status>", "smart_person", ())]);
        simple_graph.write(vec![Quad::new("<greg>", "<status>", "smart_person", ())]);

    }

    

    let g = simple_graph.g();


    /////////////////////////
    // show a simple save optional
    /////////////////////////

    let mut r:Vec<String> = g.v(vec!["<bob>", "<charle>"])
        .out("<follows>", None)
        .save_opt("<status>", "somecool")
        .all().map(|x| x["somecool"].to_string()).collect();

    let mut f:Vec<String> = vec![
        "cool_person".into(),
        "cool_person".into()
    ];

    r.sort();
    f.sort();

    println!("{:?} {:?}", r, f);

    assert_eq!(r, f);

    let mut r:Vec<HashMap<String, Value>> = g.v(vec!["<bob>", "<charle>"])
    .out("<follows>", None)
    .save_opt("<status>", "somecool")
    .all().collect();

    println!("{:?}", r);


}