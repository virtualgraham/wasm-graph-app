use gizmo_graph_db::query::gizmo;
use gizmo_graph_db::graph::quad::Quad;

use gizmo_graph_db::graph::value::Value;
use std::collections::HashMap;

fn sort_and_compare(a:&mut Vec<String>, b:&mut Vec<String>) -> bool {
    a.sort();
    b.sort();
    a == b
}

#[test]
fn simple_query_tests() {
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
    
    /////////////////////////
    // get a single vertex
    /////////////////////////
    
    let mut r:Vec<String> = g
        .v("<alice>")
        .all_values().map(|v| v.to_string()).collect();

    let mut f:Vec<String> = vec![
        "<alice>".into()
    ];

    r.sort();
    f.sort();

    assert_eq!(r, f);
    
    ///////////////////////
    // use .getLimit
    ///////////////////////

    // let mut r:Vec<String> = g
    //     .v(None)
    //     .get_limit_values(5)
    //     .map(|v| v.to_string()).collect();
    // let mut f:Vec<String> = vec![
    //     "<alice>".into(),
    //     "<bob>".into(),
    //     "<follows>".into(),
    //     "<fred>".into(),
    //     "<status>".into()
    // ];
    // r.sort();
    // f.sort();

    // assert_eq!(r, f);

    /////////////////////////
    // use .out()
    /////////////////////////
     
    let mut r:Vec<String> = g
        .v("<alice>")
        .out("<follows>", None)
        .all_values().map(|v| v.to_string()).collect();

    let mut f:Vec<String> = vec![
        "<bob>".into()
    ];

    assert!(sort_and_compare(&mut r, &mut f));


    /////////////////////////
    // use .out() (any)
    /////////////////////////
     
    let mut r:Vec<String> = g
        .v("<bob>")
        .out(None, None)
        .all_values().map(|v| v.to_string()).collect();

    let mut f:Vec<String> = vec![
        "<fred>".into(),
        "cool_person".into()
    ];

    assert!(sort_and_compare(&mut r, &mut f));


    /////////////////////////
    // use .in()
    /////////////////////////
     
    let mut r:Vec<String> = g
        .v("<bob>")
        .r#in("<follows>", None)
        .all_values().map(|v| v.to_string()).collect();

    let mut f:Vec<String> = vec![
        "<alice>".into(),
        "<charlie>".into(),
        "<dani>".into(),
    ];

    assert!(sort_and_compare(&mut r, &mut f));


    /////////////////////////
    // use .in() (any)
    /////////////////////////
     
    let mut r:Vec<String> = g
        .v("<bob>")
        .r#in(None, None)
        .all_values().map(|v| v.to_string()).collect();

    let mut f:Vec<String> = vec![
        "<alice>".into(),
        "<charlie>".into(),
        "<dani>".into(),
    ];

    assert!(sort_and_compare(&mut r, &mut f));


    /////////////////////////
    // use .in() with .filter()
    /////////////////////////

    let mut r:Vec<String> = g
        .v("<bob>")
        .r#in("<follows>", None)
        .filter(vec![gizmo::gt("<c>"), gizmo::lt("<d>")])
        .all_values().map(|v| v.to_string()).collect();

    let mut f:Vec<String> = vec![
        "<charlie>".into()
    ];

    assert!(sort_and_compare(&mut r, &mut f));


    /////////////////////////
    // use .in() with .filter(regex)
    /////////////////////////

    let mut r:Vec<String> = g
        .v("<bob>")
        .r#in("<follows>", None)
        .filter(gizmo::regex("ar?li.*e", false))
        .all_values().map(|v| v.to_string()).collect();

    let mut f:Vec<String> = vec![];

    assert!(sort_and_compare(&mut r, &mut f));


    /////////////////////////
    // use .in() with .filter(prefix)
    /////////////////////////

    let mut r:Vec<String> = g
        .v("<bob>")
        .r#in("<follows>", None)
        .filter(gizmo::like("al%"))
        .all_values().map(|v| v.to_string()).collect();

    let mut f:Vec<String> = vec![
        "<alice>".into()
    ];

    assert!(sort_and_compare(&mut r, &mut f));

    
    /////////////////////////
    // use .in() with .filter(wildcard)
    /////////////////////////

    let mut r:Vec<String> = g
        .v("<bob>")
        .r#in("<follows>", None)
        .filter(gizmo::like("a?i%e"))
        .all_values().map(|v| v.to_string()).collect();

    let mut f:Vec<String> = vec![
        "<alice>".into()
    ];

    assert!(sort_and_compare(&mut r, &mut f));


    /////////////////////////
    // use .in() with .filter(regex)
    /////////////////////////

    let mut r:Vec<String> = g
        .v("<bob>")
        .r#in("<follows>", None)
        .filter(gizmo::regex("ar?li.*e", true))
        .all_values().map(|v| v.to_string()).collect();

    let mut f:Vec<String> = vec![
        "<charlie>".into(),
        "<alice>".into()
    ];

    assert!(sort_and_compare(&mut r, &mut f));


    /////////////////////////
    // use .in() with .filter(regex,gt)
    /////////////////////////

    let mut r:Vec<String> = g
        .v("<bob>")
        .r#in("<follows>", None)
        .filter(vec![gizmo::regex("ar?li.*e", true), gizmo::gt("<c>")])
        .all_values().map(|v| v.to_string()).collect();

    let mut f:Vec<String> = vec![
        "<charlie>".into(),
    ];

    assert!(sort_and_compare(&mut r, &mut f));


    /////////////////////////
    // use .both() with tag
    /////////////////////////

    let mut r:Vec<String> = g
        .v("<fred>")
        .both(None, Some(vec!["pred".into()]))
        .all().map(|x| x["pred"].to_string()).collect(); // just pred labels


    let mut f:Vec<String> = vec![
        "<follows>".into(),
        "<follows>".into(),
        "<follows>".into(),
    ];

    assert!(sort_and_compare(&mut r, &mut f));


    /////////////////////////
    // use .tag()-.is()-.back()
    /////////////////////////
 
    let mut r:Vec<String> = g
        .v("<bob>")
        .r#in("<follows>", None)
        .tag(vec!["foo".into()])
        .out("<status>", None)
        .is("cool_person")
        .back("foo")
        .all_values().map(|v| v.to_string()).collect();

    let mut f:Vec<String> = vec![
        "<dani>".into(),
    ];

    assert!(sort_and_compare(&mut r, &mut f));


    /////////////////////////
    // separate .tag()-.is()-.back()
    /////////////////////////
 
    let x = g
        .v("<charlie>")
        .out("<follows>", None)
        .tag(vec!["foo".into()])
        .out("<status>", None)
        .is("cool_person")
        .back("foo");

    let mut r:Vec<String> = x
        .r#in("<follows>", None)
        .is("<dani>")
        .back("foo")
        .all_values().map(|v| v.to_string()).collect();

    let mut f:Vec<String> = vec![
        "<bob>".into(),
    ];

    assert!(sort_and_compare(&mut r, &mut f));


    /////////////////////////
    // do multiple .back()
    /////////////////////////
 
    let mut r:Vec<String> = g
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
        .all().map(|x| x["acd"].to_string()).collect(); // just pred labels

    let mut f:Vec<String> = vec![
        "<dani>".into(),
    ];

    r.sort();
    f.sort();

    println!("{:?}", r);

    assert_eq!(r, f);

    /////////////////////////
    // use Except to filter out a single vertex
    /////////////////////////

    let a = g.v("<alice>").clone();

    let mut r:Vec<String> = g
        .v(vec!["<alice>".into(), "<bob>".into()])
        .except(&a)
        .all_values().map(|v| v.to_string()).collect();

    let mut f:Vec<String> = vec![
        "<bob>".into()
    ];

    assert!(sort_and_compare(&mut r, &mut f));


    /////////////////////////
    // use chained Except
    /////////////////////////

    let a = g.v("<bob>").clone();
    let b = g.v("<charlie>").clone();

    let mut r:Vec<String> = g
        .v(vec!["<alice>".into(), "<bob>".into(), "<charlie>".into()])
        .except(&a)
        .except(&b)
        .all_values().map(|v| v.to_string()).collect();

    let mut f:Vec<String> = vec![
        "<alice>".into()
    ];

    assert!(sort_and_compare(&mut r, &mut f));


    /////////////////////////
    // show simple morphism
    /////////////////////////

    let grandfollows = g.m().out("<follows>", None).out("<follows>", None).clone();

    let mut r:Vec<String> = g
        .v("<charlie>")
        .follow(&grandfollows)
        .all_values().map(|v| v.to_string()).collect();

    let mut f:Vec<String> = vec![
        "<greg>".into(),
        "<fred>".into(),
        "<bob>".into()
    ];

    assert!(sort_and_compare(&mut r, &mut f));


    /////////////////////////
    // show reverse morphism
    /////////////////////////

    let grandfollows = g.m().out("<follows>", None).out("<follows>", None).clone();

    let mut r:Vec<String> = g
        .v("<fred>")
        .follow_r(&grandfollows)
        .all_values().map(|v| v.to_string()).collect();

    let mut f:Vec<String> = vec![
        "<alice>".into(),
        "<charlie>".into(),
        "<dani>".into()
    ];

    assert!(sort_and_compare(&mut r, &mut f));


    /////////////////////////
    // show simple intersection
    /////////////////////////

    fn follows(g: &mut gizmo::Graph, x: &str) -> gizmo::Path {
        g.v(x).out("<follows>", None).clone()
    }

    let mut r:Vec<String> = follows(g, "<dani>")
        .and(&follows(g, "<charlie>"))
        .all_values().map(|v| v.to_string()).collect();

    let mut f:Vec<String> = vec![
        "<bob>".into(),
    ];

    assert!(sort_and_compare(&mut r, &mut f));


    /////////////////////////
    // show simple morphism intersection
    /////////////////////////

    fn gfollows(g: &mut gizmo::Graph, x: &str) -> gizmo::Path {
        let grandfollows = g.m().out("<follows>", None).out("<follows>", None).clone();
        g.v(x).follow(&grandfollows).clone()
    }

    let mut r:Vec<String> = gfollows(g, "<alice>")
        .and(&gfollows(g, "<charlie>"))
        .all_values().map(|v| v.to_string()).collect();

    let mut f:Vec<String> = vec![
        "<fred>".into(),
    ];

    assert!(sort_and_compare(&mut r, &mut f));


    /////////////////////////
    // show double morphism intersection
    /////////////////////////

    let mut r:Vec<String> = gfollows(g, "<emily>")
        .and(&gfollows(g, "<charlie>"))
        .and(&gfollows(g, "<bob>"))
        .all_values().map(|v| v.to_string()).collect();

    let mut f:Vec<String> = vec![
        "<greg>".into(),
    ];

    assert!(sort_and_compare(&mut r, &mut f));


    /////////////////////////
    // show reverse intersection
    /////////////////////////

    let grandfollows = g.m().out("<follows>", None).out("<follows>", None).clone();
    let s = g.v("<fred>").follow_r(&grandfollows).clone();

    let mut r:Vec<String> = g.v("<greg>")
        .follow_r(&grandfollows)
        .intersect(&s)
        .all_values().map(|v| v.to_string()).collect();

    let mut f:Vec<String> = vec![
        "<charlie>".into(),
    ];

    assert!(sort_and_compare(&mut r, &mut f));
}
