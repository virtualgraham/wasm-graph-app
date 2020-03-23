use gizmo_graph_db::query::gizmo;
use gizmo_graph_db::graph::quad::Quad;
use gizmo_graph_db::graph::value::Value;
use std::collections::HashMap;

#[test]
fn simple_query_tests() {
    let mut simple_graph = gizmo::new_memory_graph();

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

    }

    let g = simple_graph.g();

    // get a single vertex
    let r:Vec<HashMap<String, Value>> = g.v("<alice>").all().collect();
    let f:Vec<HashMap<String, Value>> = vec![hashmap!{"id".into() => "<alice>".into()}];
    assert_eq!(r, f);
    

    // // use .getLimit
    // let r:Vec<HashMap<String, Value>> = g.v(None).get_limit(5).collect();
    // let f:Vec<HashMap<String, Value>> = vec![
    //     hashmap!{"id".into() => "<alice>".into()},
    //     hashmap!{"id".into() => "<bob>".into()},
    //     hashmap!{"id".into() => "<follows>".into()},
    //     hashmap!{"id".into() => "<fred>".into()},
    //     hashmap!{"id".into() => "<status>".into()}
    // ];
    // assert_eq!(r, f);
    


    // let r:Vec<HashMap<String, Value>> = g.v("<alice>").out("<follows>", None).all().collect();
    
    // let r:Vec<HashMap<String, Value>> = g.v("<bob>").out(None, None).all().collect();

    //let r:Vec<HashMap<String, Value>> = g.v("<bob>").r#in("<follows>", None).all().collect();

    //let r:Vec<HashMap<String, Value>> = g.v("<fred>").both("<follows>", None).all().collect();

 
    // regex filter
    let r:Vec<HashMap<String, Value>> = g.v("<bob>").r#in("<follows>", None).filter(gizmo::regex("ar?li.*e")).all().collect();
    let f:Vec<HashMap<String, Value>> = vec![
        hashmap!{"id".into() => "<charlie>".into()}, 
        hashmap!{"id".into() => "<alice>".into()}
    ];
    assert_eq!(r, f);

}