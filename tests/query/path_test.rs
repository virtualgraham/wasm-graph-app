use gizmo_graph_db::query::path::*;
use gizmo_graph_db::graph::iterator::iterate::EachIterator;
use gizmo_graph_db::graph::quad::{QuadStore, Quad, QuadWriter, IgnoreOptions};
use gizmo_graph_db::graph::value::Value;
use gizmo_graph_db::graph::memstore::quadstore::MemStore;

use std::rc::Rc;
use std::cell::RefCell;

fn run_top_level(qs: Rc<RefCell<dyn QuadStore>>, path: &Path, opt: bool) -> Vec<String> {
    EachIterator::new(path.build_iterator_on(qs.clone()), false, true).filter_map(move |r| qs.borrow().name_of(&r)).map(|v| v.to_string()).collect()
}

fn path_tests() {

    let v_empty = Value::from("");

    let v_follows = Value::from("<follows>");
    let v_are= Value::from("<are>");
    let v_status= Value::from("<status>");
    let v_predicate = Value::from("<predicates>");

    let v_cool = Value::from("cool_person");
    let v_smart = Value::from("smart_person");
    let v_smart_graph = Value::from("<smart_graph>");

    let v_alice = Value::from("<alice>");
    let v_bob = Value::from("<bob>");
    let v_charlie= Value::from("<charlie>");
    let v_dani = Value::from("<dani>");
    let v_fred = Value::from("<fred>");
    let v_greg= Value::from("<greg>");
    let v_emily = Value::from("<emily>");

    let qs = Rc::new(RefCell::new(MemStore::new()));

    let qw = QuadWriter::new(qs.clone(), IgnoreOptions{ignore_dup: true, ignore_missing: true});

    qw.add_quad(Quad::new("<alice>", "<follows>", "<bob>", ()));
    qw.add_quad(Quad::new("<bob>", "<follows>", "<fred>", ()));
    qw.add_quad(Quad::new("<bob>", "<status>", "cool_person", ()));

    qw.add_quad(Quad::new("<dani>", "<follows>", "<bob>", ()));
    qw.add_quad(Quad::new("<charlie>", "<follows>", "<bob>", ()));
    qw.add_quad(Quad::new("<charlie>", "<follows>", "<dani>", ()));

    qw.add_quad(Quad::new("<dani>", "<follows>", "<greg>", ()));
    qw.add_quad(Quad::new("<dani>", "<status>", "cool_person", ()));
    qw.add_quad(Quad::new("<emily>", "<follows>", "<fred>", ()));

    qw.add_quad(Quad::new("<fred>", "<follows>", "<greg>", ()));
    qw.add_quad(Quad::new("<greg>", "<status>", "cool_person", ()));
    qw.add_quad(Quad::new("<predicates>", "<are>", "<follows>", ()));

    qw.add_quad(Quad::new("<predicates>", "<are>", "<status>", ()));
    qw.add_quad(Quad::new("<emily>", "<status>", "smart_person", "<smart_graph>"));
    qw.add_quad(Quad::new("<greg>", "<status>", "smart_person", "<smart_graph>"));

    //////////////////
    // out
    //////////////////

    let path = Path::start_path(Some(qs.clone()), vec![Value::from("<alice>")]);
    let mut got = run_top_level(qs.clone(), &path, false);
    let mut expect = vec![
        "<bob>".to_string()
    ];

    got.sort();
    expect.sort();

    assert_eq!(got, expect);

    //////////////////
    // out (any)
    //////////////////

    let path = Path::start_path(Some(qs.clone()), vec![Value::from("<bob>")]);
    let mut got = run_top_level(qs.clone(), &path, false);
    let mut expect = vec![
        "<fred>".to_string(),
        "cool_person".to_string()
    ];

    got.sort();
    expect.sort();

    assert_eq!(got, expect);
}