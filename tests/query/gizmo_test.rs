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
        .both(None, "pred")
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
        .tag("foo")
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
 
    let mut x = g
        .v("<charlie>")
        .out("<follows>", None)
        .tag("foo")
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
        .r#as("f")
        .out("<follows>", None)
        .out("<status>", None)
        .is("cool_person")
        .back("f")
        .r#in("<follows>", None)
        .r#in("<follows>", None)
        .r#as("acd")
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

    let mut r:Vec<String> = g
        .v(vec!["<alice>", "<bob>"])
        .except(&g.v("<alice>"))
        .all_values().map(|v| v.to_string()).collect();

    let mut f:Vec<String> = vec![
        "<bob>".into()
    ];

    assert!(sort_and_compare(&mut r, &mut f));


    /////////////////////////
    // use chained Except
    /////////////////////////

    let mut r:Vec<String> = g
        .v(vec!["<alice>", "<bob>", "<charlie>"])
        .except(&g.v("<bob>"))
        .except(&g.v("<charlie>"))
        .all_values().map(|v| v.to_string()).collect();

    let mut f:Vec<String> = vec![
        "<alice>".into()
    ];

    assert!(sort_and_compare(&mut r, &mut f));


    /////////////////////////
    // show simple morphism
    /////////////////////////

    let grandfollows = g.m().out("<follows>", None).out("<follows>", None);

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

    let grandfollows = g.m().out("<follows>", None).out("<follows>", None);

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

    let follows = |x: &str| g.v(x).out("<follows>", None);

    let mut r:Vec<String> = follows("<dani>")
        .and(&follows("<charlie>"))
        .all_values().map(|v| v.to_string()).collect();

    let mut f:Vec<String> = vec![
        "<bob>".into(),
    ];

    assert!(sort_and_compare(&mut r, &mut f));


    /////////////////////////
    // show simple morphism intersection
    /////////////////////////

    let grandfollows = g.m().out("<follows>", None).out("<follows>", None);

    let gfollows = |x: &str| {
        g.v(x).follow(&grandfollows)
    };

    let mut r:Vec<String> = gfollows("<alice>")
        .and(&gfollows("<charlie>"))
        .all_values().map(|v| v.to_string()).collect();

    let mut f:Vec<String> = vec![
        "<fred>".into(),
    ];

    assert!(sort_and_compare(&mut r, &mut f));


    /////////////////////////
    // show double morphism intersection
    /////////////////////////

    let mut r:Vec<String> = gfollows("<emily>")
        .and(&gfollows("<charlie>"))
        .and(&gfollows("<bob>"))
        .all_values().map(|v| v.to_string()).collect();

    let mut f:Vec<String> = vec![
        "<greg>".into(),
    ];

    assert!(sort_and_compare(&mut r, &mut f));


    /////////////////////////
    // show reverse intersection
    /////////////////////////

    let grandfollows = g.m().out("<follows>", None).out("<follows>", None);

    let mut r:Vec<String> = g.v("<greg>")
        .follow_r(&grandfollows)
        .intersect(&g.v("<fred>").follow_r(&grandfollows))
        .all_values().map(|v| v.to_string()).collect();

    let mut f:Vec<String> = vec![
        "<charlie>".into(),
    ];

    assert!(sort_and_compare(&mut r, &mut f));



    /////////////////////////
    // show standard sort of morphism intersection, continue follow
    /////////////////////////

    let gfollowers = g.m().r#in("<follows>", None).r#in("<follows>", None);
    
    let cool = |x: &str| g.v(x).r#as("a").out("<status>", None).is("cool_person").back("a");

    let mut r:Vec<String> = cool("<greg>")
        .follow(&gfollowers)
        .intersect(&cool("<bob>"))
        .follow(&gfollowers)
        .all_values().map(|v| v.to_string()).collect();

    let mut f:Vec<String> = vec![
        "<charlie>".into(),
    ];

    assert!(sort_and_compare(&mut r, &mut f));


    /////////////////////////
    // test Or()
    /////////////////////////

    let mut r:Vec<String> = g.v("<bob>")
        .out("<follows>", None)
        .or(&g.v(None).has("<status>", "cool_person"))
        .all_values().map(|v| v.to_string()).collect();

    let mut f:Vec<String> = vec![
        "<fred>".into(),
        "<bob>".into(),
        "<greg>".into(),
        "<dani>".into(),
    ];

    assert!(sort_and_compare(&mut r, &mut f));


    /////////////////////////
    // show a simple Has
    /////////////////////////

    let mut r:Vec<String> = g.v(None)
        .has("<status>", "cool_person")
        .all_values().map(|v| v.to_string()).collect();

    let mut f:Vec<String> = vec![
        "<greg>".into(),
        "<dani>".into(),
        "<bob>".into()
    ];

    assert!(sort_and_compare(&mut r, &mut f));


    /////////////////////////
    // show a simple HasR
    /////////////////////////

    let mut r:Vec<String> = g.v(None)
        .has_r("<status>", "<bob>")
        .all_values().map(|v| v.to_string()).collect();

    let mut f:Vec<String> = vec![
        "cool_person".into()
    ];

    assert!(sort_and_compare(&mut r, &mut f));

    /////////////////////////
    // show a double Has
    /////////////////////////

    let mut r:Vec<String> = g.v(None)
        .has("<status>", "cool_person")
        .has("<follows>", "<fred>")
        .all_values().map(|v| v.to_string()).collect();

    let mut f:Vec<String> = vec![
        "<bob>".into()
    ];

    assert!(sort_and_compare(&mut r, &mut f));

    /////////////////////////
    // show a Has with filter
    /////////////////////////

    let mut r:Vec<String> = g.v(None)
        .has("<follows>", gizmo::gt("<f>"))
        .all_values().map(|v| v.to_string()).collect();

    let mut f:Vec<String> = vec![
        "<bob>".into(),
        "<dani>".into(),
        "<emily>".into(),
        "<fred>".into()
    ];

    assert!(sort_and_compare(&mut r, &mut f));

    /////////////////////////
    // use Limit
    /////////////////////////

    let mut r:Vec<String> = g.v(None)
        .has("<status>", "cool_person")
        .limit(2)
        .all_values().map(|v| v.to_string()).collect();

    let mut f:Vec<String> = vec![
        "<bob>".into(),
        "<dani>".into()
    ];

    assert!(sort_and_compare(&mut r, &mut f));

    /////////////////////////
    // use Skip
    /////////////////////////

    let mut r:Vec<String> = g.v(None)
        .has("<status>", "cool_person")
        .skip(2)
        .all_values().map(|v| v.to_string()).collect();

    let mut f:Vec<String> = vec![
        "<greg>".into()
    ];

    assert!(sort_and_compare(&mut r, &mut f));

    /////////////////////////
    // use Skip and Limit
    /////////////////////////

    let mut r:Vec<String> = g.v(None)
        .has("<status>", "cool_person")
        .skip(1)
        .limit(1)
        .all_values().map(|v| v.to_string()).collect();

    let mut f:Vec<String> = vec![
        "<dani>".into()
    ];

    assert!(sort_and_compare(&mut r, &mut f));

    /////////////////////////
    // show Count
    /////////////////////////

    let  c = g.v(None)
        .has("<status>", None)
        .count();

    assert_eq!(c, 5);

    /////////////////////////
    // show a simple save
    /////////////////////////

    let mut r:Vec<String> = g.v(None)
        .save("<status>", "somecool")
        .all().map(|x| x["somecool"].to_string()).collect();

    let mut f:Vec<String> = vec![
        "cool_person".into(),
        "cool_person".into(),
        "cool_person".into(),
        "smart_person".into(),
        "smart_person".into(),
    ];

    assert!(sort_and_compare(&mut r, &mut f));

    /////////////////////////
    // show a simple save optional
    /////////////////////////

    // let mut r:Vec<String> = g.v(vec!["<bob>", "<charle>"])
    //     .out("<follows>", None)
    //     .save_opt("<status>", "somecool")
    //     .all().map(|x| x["somecool"].to_string()).collect();

    // let mut f:Vec<String> = vec![
    //     "cool_person".into(),
    //     "cool_person".into()
    // ];

    // r.sort();
    // f.sort();

    // assert_eq!(r, f);

    /////////////////////////
    // save iri no tag
    /////////////////////////

    let mut r:Vec<String> = g.v(None)
        .save("<status>", None)
        .all().map(|x| x["<status>"].to_string()).collect();

    let mut f:Vec<String> = vec![
        "cool_person".into(),
        "cool_person".into(),
        "cool_person".into(),
        "smart_person".into(),
        "smart_person".into(),
    ];

    r.sort();
    f.sort();

    assert_eq!(r, f);


    /////////////////////////
    // show a simple saveR
    /////////////////////////

    let mut r:Vec<String> = g.v("cool_person")
        .save_r("<status>", "who")
        .all().map(|x| x["who"].to_string()).collect();

    let mut f:Vec<String> = vec![
        "<greg>".into(),
        "<dani>".into(),
        "<bob>".into()
    ];

    assert!(sort_and_compare(&mut r, &mut f));

    /////////////////////////
    // show an out save
    /////////////////////////

    let mut r:Vec<String> = g.v("<dani>")
        .out(None, "pred")
        .all().map(|x| x["pred"].to_string()).collect();

    let mut f:Vec<String> = vec![
        "<follows>".into(),
        "<follows>".into(),
        "<status>".into()
    ];

    assert!(sort_and_compare(&mut r, &mut f));

    /////////////////////////
    // show a tag list
    /////////////////////////

    let mut r:Vec<String> = g.v("<dani>")
        .out(None, vec!["pred", "foo", "bar"])
        .all().map(|x| x["foo"].to_string()).collect();

    let mut f:Vec<String> = vec![
        "<follows>".into(),
        "<follows>".into(),
        "<status>".into()
    ];

    assert!(sort_and_compare(&mut r, &mut f));

    /////////////////////////
    // show a pred list
    /////////////////////////

    let mut r:Vec<String> = g.v("<dani>")
        .out(vec!["<follows>".into(), "<status>".into()], None)
        .all_values().map(|v| v.to_string()).collect();

    let mut f:Vec<String> = vec![
        "<bob>".into(),
        "<greg>".into(),
        "cool_person".into()
    ];

    assert!(sort_and_compare(&mut r, &mut f));


    /////////////////////////
    // show a predicate path
    /////////////////////////

    let mut r:Vec<String> = g.v("<dani>")
        .out(&g.v("<follows>"), "pred")
        .all_values().map(|v| v.to_string()).collect();

    let mut f:Vec<String> = vec![
        "<bob>".into(),
        "<greg>".into()
    ];

    assert!(sort_and_compare(&mut r, &mut f));


    /////////////////////////
    // list all bob's incoming predicates
    /////////////////////////

    let mut r:Vec<String> = g.v("<bob>")
        .in_predicates()
        .all_values().map(|v| v.to_string()).collect();

    let mut f:Vec<String> = vec![
        "<follows>".into()
    ];

    assert!(sort_and_compare(&mut r, &mut f));


    /////////////////////////
    // save all bob's incoming predicates
    /////////////////////////

    let mut r:Vec<String> = g.v("<bob>")
        .save_in_predicates("pred")
        .all().map(|x| x["pred"].to_string()).collect();

    let mut f:Vec<String> = vec![
        "<follows>".into(),
        "<follows>".into(),
        "<follows>".into()
    ];

    assert!(sort_and_compare(&mut r, &mut f));

    /////////////////////////
    // list all labels
    /////////////////////////

    let mut r:Vec<String> = g.v(None)
        .labels()
        .all_values().map(|v| v.to_string()).collect();

    let mut f:Vec<String> = vec![
        "<smart_graph>".into()
    ];

    assert!(sort_and_compare(&mut r, &mut f));


    /////////////////////////
    // list all in predicates
    /////////////////////////

    let mut r:Vec<String> = g.v(None)
        .in_predicates()
        .all_values().map(|v| v.to_string()).collect();

    let mut f:Vec<String> = vec![
        "<are>".into(),
        "<follows>".into(),
        "<status>".into()
    ];

    assert!(sort_and_compare(&mut r, &mut f));


    /////////////////////////
    // list all out predicates
    /////////////////////////

    let mut r:Vec<String> = g.v(None)
        .out_predicates()
        .all_values().map(|v| v.to_string()).collect();

    let mut f:Vec<String> = vec![
        "<are>".into(),
        "<follows>".into(),
        "<status>".into()
    ];

    assert!(sort_and_compare(&mut r, &mut f));


    /////////////////////////
    // traverse using LabelContext
    /////////////////////////

    let mut r:Vec<String> = g.v("<greg>")
        .label_context("<smart_graph>", None)
        .out("<status>", None)
        .all_values().map(|v| v.to_string()).collect();

    let mut f:Vec<String> = vec![
        "smart_person".into(),
    ];

    assert!(sort_and_compare(&mut r, &mut f));





}

