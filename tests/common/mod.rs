use gizmo_graph_db::graph::iterator::fixed::{Fixed};
use std::rc::Rc;
use std::cell::RefCell;
use gizmo_graph_db::graph::iterator::{Shape, Scanner, Index, Costs, Base, ShapeType};
use gizmo_graph_db::graph::refs;
use gizmo_graph_db::graph::value::Value;
use std::collections::HashMap;
use std::fmt;

pub struct Test {
    shape: Rc<RefCell<dyn Shape>>,
    next: bool,
    err: Option<String>
}

impl Test {
    pub fn new(next:bool, err: Option<String>) -> Rc<RefCell<Test>> {
        Rc::new(RefCell::new(Test {
            shape: Fixed::new(vec![]),
            next,
            err
        }))
    }
}


impl fmt::Display for Test {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Test")
    }
}


impl Shape for Test {
    fn iterate(&self) -> Rc<RefCell<dyn Scanner>> {
        Rc::new(RefCell::new(TestNext {
            scanner: self.shape.borrow().iterate(),
            next: self.next,
            err: self.err.clone()
        }))
    }
    fn lookup(&self) -> Rc<RefCell<dyn Index>> {
        Rc::new(RefCell::new(TestContains {
            index: self.shape.borrow().lookup(),
            next: self.next,
            err: self.err.clone()
        }))
    }
    fn stats(&mut self) -> Result<Costs, String> {
        self.shape.borrow_mut().stats()
    }
    fn optimize(&mut self) -> Option<Rc<RefCell<dyn Shape>>> {
        self.shape.borrow_mut().optimize()
    }
    fn sub_iterators(&self) -> Option<Vec<Rc<RefCell<dyn Shape>>>> {
        self.shape.borrow().sub_iterators()
    }
    fn shape_type(&mut self) -> ShapeType {
        ShapeType::Test
    }
}


struct TestNext {
    scanner: Rc<RefCell<dyn Scanner>>,
    next: bool,
    err: Option<String>
}

impl fmt::Display for TestNext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TestNext")
    }
}

impl Base for TestNext {
    fn tag_results(&self, tags: &mut HashMap<String, refs::Ref>) {
        return self.scanner.borrow().tag_results(tags)
    }
    fn result(&self) -> Option<refs::Ref> {
        return self.scanner.borrow().result()
    }
    fn next_path(&mut self) -> bool {
        return self.scanner.borrow_mut().next_path()
    }
    fn err(&self) -> Option<String> {
        return self.err.clone()
    }
    fn close(&mut self) -> Result<(), String> {
        return self.scanner.borrow_mut().close()
    }
}

impl Scanner for TestNext {
    #[allow(unused)]
    fn next(&mut self) -> bool {
        return self.next
    }
}


struct TestContains {
    index: Rc<RefCell<dyn Index>>,
    next: bool,
    err: Option<String>
}

impl fmt::Display for TestContains {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "TestContains")
    }
}

impl Base for TestContains {
    fn tag_results(&self, tags: &mut HashMap<String, refs::Ref>) {
        return self.index.borrow().tag_results(tags)
    }
    fn result(&self) -> Option<refs::Ref> {
        return self.index.borrow().result()
    }
    fn next_path(&mut self) -> bool {
        return self.index.borrow_mut().next_path()
    }
    fn err(&self) -> Option<String> {
        return self.err.clone()
    }
    fn close(&mut self) -> Result<(), String> {
        return self.index.borrow_mut().close()
    }
}

impl Index for TestContains {
    #[allow(unused)]
    fn contains(&mut self, v:&refs::Ref) -> bool {
        return self.next
    }
}





pub struct Int64 {
    node: bool,
    min: i64,
    max: i64
}

impl Int64 {
    pub fn new(min: i64, max: i64, node: bool) -> Rc<RefCell<Int64>> {
        Rc::new(RefCell::new(Int64 {
            min,
            max, 
            node
        }))
    }

    fn size(&self) -> refs::Size {
        refs::Size {
            value: (self.max - self.min) + 1,
            exact: true
        }
    }
}


impl fmt::Display for Int64 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Int64")
    }
}


impl Shape for Int64 {
    fn iterate(&self) -> Rc<RefCell<dyn Scanner>> {
        Rc::new(RefCell::new(Int64Next {
            node: self.node,
            min: self.min,
            max: self.max,
            at: self.min,
            result: 0
        }))
    }

    fn lookup(&self) -> Rc<RefCell<dyn Index>> {
        Rc::new(RefCell::new(Int64Contains {
            node: self.node,
            min: self.min,
            max: self.max,
            at: self.min,
            result: 0
        }))
    }

    #[allow(unused)]
    fn stats(&mut self) -> Result<Costs, String> {
        let s = self.size();
        return Ok(Costs {
            contains_cost: 1,
            next_cost: 1,
            size: s
        })
    }

    #[allow(unused)]
    fn optimize(&mut self) -> Option<Rc<RefCell<dyn Shape>>> {
        None
    }

    fn sub_iterators(&self) -> Option<Vec<Rc<RefCell<dyn Shape>>>> {
        None
    }

    fn shape_type(&mut self) -> ShapeType {
        ShapeType::Int64
    }
}

fn val_to_int_64(v: &refs::Ref) -> i64 {
    if let refs::Content::Value(c) = &v.content {
        if let Value::Number(n) = c {
            return n.as_i64().unwrap()
        }
    }
    panic!("Not i64 value")
}

struct Int64Next {
    node: bool,
    min: i64,
    max: i64,
    at: i64,
    result: i64
}

impl fmt::Display for Int64Next {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Int64Next")
    }
}

impl Base for Int64Next {

    #[allow(unused)]
    fn tag_results(&self, tags: &mut HashMap<String, refs::Ref>) {}

    fn result(&self) -> Option<refs::Ref> {
        // TODO: NODE AND QUAD
        Some(refs::Ref::new_i64_node(self.result))
    }

    #[allow(unused)]
    fn next_path(&mut self) -> bool {
        false
    }

    fn err(&self) -> Option<String> {
        None
    }

    fn close(&mut self) -> Result<(), String> {
        Ok(())
    }
}

impl Scanner for Int64Next {
    #[allow(unused)]
    fn next(&mut self) -> bool {
        if self.at == -1 {
            return false
        }
        let val = self.at;
        self.at = self.at + 1;
        if self.at > self.max {
            self.at = -1;
        }
        self.result = val;
        return true
    }
}


struct Int64Contains {
    node: bool,
    min: i64,
    max: i64,
    at: i64,
    result: i64
}

impl fmt::Display for Int64Contains {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Int64Contains")
    }
}

impl Base for Int64Contains {

    #[allow(unused)]
    fn tag_results(&self, tags: &mut HashMap<String, refs::Ref>) {}

    fn result(&self) -> Option<refs::Ref> {
        // TODO: NODE AND QUAD
        Some(refs::Ref::new_i64_node(self.result))
    }

    #[allow(unused)]
    fn next_path(&mut self) -> bool {
        false
    }

    fn err(&self) -> Option<String> {
        None
    }

    fn close(&mut self) -> Result<(), String> {
        Ok(())
    }
}

impl Index for Int64Contains {
    #[allow(unused)]
    fn contains(&mut self, v:&refs::Ref) -> bool {
        let v = val_to_int_64(v);
        if self.min <= v && v <= self.max {
            self.result = v;
            return true
        }
        return false
    }
}


pub fn iterated(s: Rc<RefCell<dyn Shape>>) -> Vec<i64> {
    let mut res = Vec::new();
    let it = s.borrow().iterate();
    while it.borrow_mut().next() {

        let n = val_to_int_64(it.borrow().result().as_ref().unwrap());
        res.push(n)
    }
    let _ = it.borrow_mut().close();
    return res
}