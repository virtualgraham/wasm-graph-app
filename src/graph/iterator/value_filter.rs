use super::{Shape, ShapeType, Base, Index, Scanner, Costs};
use super::super::refs;
use super::super::value::{Value};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use io_context::Context;
use std::fmt;
use super::super::quad::QuadStore;
use serde_json::value::Number;
use regex::Regex;




pub struct RegexValueFilter {
    re: Regex
}

impl RegexValueFilter {
    pub fn new(sub: Rc<RefCell<dyn Shape>>, qs: Rc<RefCell<dyn QuadStore>>, re: Regex) -> Rc<RefCell<ValueFilter>> {
        ValueFilter::new(
            qs, 
            sub, 
            Rc::new(RegexValueFilter {
                re
            })
        )
    }
}

impl ValueFilterFunction for RegexValueFilter {

    fn filter(&self, qval: Value) -> Result<bool, String> {
        match qval {
            Value::String(s) => {
                Ok(self.re.is_match(&s))
            },
            _ => { return Ok(false) }
        }
    }

}



#[derive(Clone)]
pub enum Operator {
    LT,
    LTE,
    GT,
    GTE
}



pub struct ComparisonValueFilter {
    op:Operator,
    val: Value, 
}

impl ComparisonValueFilter {
    pub fn new(sub: Rc<RefCell<dyn Shape>>, op:Operator, val: Value, qs: Rc<RefCell<dyn QuadStore>>) -> Rc<RefCell<ValueFilter>> {
        ValueFilter::new(
            qs, 
            sub, 
            Rc::new(ComparisonValueFilter {
                op,
                val
            })
        )
    }
}

impl ValueFilterFunction for ComparisonValueFilter {
    fn filter(&self, qval: Value) -> Result<bool, String> {
        match &self.val {
            Value::String(a) => {
                if let Value::String(b) = qval {
                    return Ok(run_str_op(a, &self.op, &b))
                } else {
                    return Ok(false)
                }
            },
            Value::Number(a) => {
                if let Value::Number(b) = qval {
                    return Ok(run_number_op(a, &self.op, &b))
                } else {
                    return Ok(false)
                }
            },
            _ => return Ok(run_str_op(&self.val.to_string(), &self.op, &qval.to_string()))
        }
    }
}


fn run_str_op(a: &String, op:&Operator, b:&String) -> bool {
    return match op {
        Operator::LT => a < b,
        Operator::GT => a > b,
        Operator::LTE => a <= b,
        Operator::GTE => a >= b,
    }
}

fn run_number_op(a: &Number, op:&Operator, b: &Number) -> bool {
    if a.is_f64() {

        if !b.is_f64() { return false }

        return match op {
            Operator::LT => a.as_f64() < b.as_f64(),
            Operator::GT => a.as_f64() > b.as_f64(),
            Operator::LTE => a.as_f64() <= b.as_f64(),
            Operator::GTE => a.as_f64() >= b.as_f64(),
        }

    } else if a.is_i64() {

        if !b.is_i64() { return false }

        return match op {
            Operator::LT => a.as_i64() < b.as_i64(),
            Operator::GT => a.as_i64() > b.as_i64(),
            Operator::LTE => a.as_i64() <= b.as_i64(),
            Operator::GTE => a.as_i64() >= b.as_i64(),
        }

    } else if a.is_u64() {

        if !b.is_u64() { return false }

        return match op {
            Operator::LT => a.as_u64() < b.as_u64(),
            Operator::GT => a.as_u64() > b.as_u64(),
            Operator::LTE => a.as_u64() <= b.as_u64(),
            Operator::GTE => a.as_u64() >= b.as_u64(),
        }

    }

    return false
}

pub trait ValueFilterFunction {
    fn filter(&self, v: Value) -> Result<bool, String>;
}

pub struct ValueFilter {
    sub: Rc<RefCell<dyn Shape>>,
    filter: Rc<dyn ValueFilterFunction>,
    qs: Rc<RefCell<dyn QuadStore>>
}

impl ValueFilter {
    pub fn new(qs: Rc<RefCell<dyn QuadStore>>, sub: Rc<RefCell<dyn Shape>>, filter: Rc<dyn ValueFilterFunction>) -> Rc<RefCell<ValueFilter>> {
        Rc::new(RefCell::new( ValueFilter {
            sub,
            filter, 
            qs
        }))
    }
}

impl fmt::Display for ValueFilter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ValueFilter")
    }
}


impl Shape for ValueFilter {

    fn iterate(&self) -> Rc<RefCell<dyn Scanner>> {
        ValueFilterNext::new(self.qs.clone(), self.sub.borrow().iterate(), self.filter.clone())
    }

    fn lookup(&self) -> Rc<RefCell<dyn Index>> {
        ValueFilterContains::new(self.qs.clone(), self.sub.borrow().lookup(), self.filter.clone())
    }

    fn stats(&mut self, ctx: &Context) -> Result<Costs, String> {
        let mut st = self.sub.borrow_mut().stats(ctx)?;
        st.size.value = st.size.value/2 + 1;
        st.size.exact = false;
        return Ok(st);
    }

    fn optimize(&mut self, ctx: &Context) -> Option<Rc<RefCell<dyn Shape>>> {
        let new_sub = self.sub.borrow_mut().optimize(ctx);
        if new_sub.is_some() {
            self.sub = new_sub.unwrap();
        }
        return None
    }

    fn sub_iterators(&self) -> Option<Vec<Rc<RefCell<dyn Shape>>>> {
        Some(vec![self.sub.clone()])
    }

    fn shape_type(&mut self) -> ShapeType {
        ShapeType::ValueFilter
    }
}



struct ValueFilterNext {
    sub: Rc<RefCell<dyn Scanner>>,
    filter: Rc<dyn ValueFilterFunction>,
    qs: Rc<RefCell<dyn QuadStore>>,
    result: Option<refs::Ref>,
    err: Option<String>
}

impl ValueFilterNext {
    fn new(qs: Rc<RefCell<dyn QuadStore>>, sub: Rc<RefCell<dyn Scanner>>, filter: Rc<dyn ValueFilterFunction>) -> Rc<RefCell<ValueFilterNext>> {
       Rc::new(RefCell::new( ValueFilterNext {
           sub,
           filter,
           qs,
           result: None,
           err: None
       }))
    }

    fn do_filter(&mut self, val: &refs::Ref) -> bool {
        let qval = self.qs.borrow().name_of(val);
        if qval.is_none() {
            self.err = Some("no name for val".to_string());
            return false
        }
        let res = self.filter.filter(qval.unwrap());
        match res {
            Result::Ok(r) => r,
            Result::Err(e) => {
                self.err = Some(e);
                return false
            }
        }
    }
}

impl fmt::Display for ValueFilterNext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ValueFilterNext")
    }
}

impl Base for ValueFilterNext {
    fn tag_results(&self, tags: &mut HashMap<String, refs::Ref>) {
        self.sub.borrow().tag_results(tags)
    }

    fn result(&self) -> Option<refs::Ref> {
        return self.result.clone()
    }

    fn next_path(&mut self, ctx: &Context) -> bool {
        return self.sub.borrow_mut().next_path(ctx)
    }

    fn err(&self) -> Option<String> {
        return self.err.clone()
    }

    fn close(&mut self) -> Result<(), String> {
        return self.sub.borrow_mut().close()
    }
}

impl Scanner for ValueFilterNext {
    fn next(&mut self, ctx: &Context) -> bool {
        while self.sub.borrow_mut().next(ctx) {
            let val = self.sub.borrow().result().unwrap();
            if self.do_filter(&val) {
                self.result = Some(val);
                return true
            }
        }
        self.err = self.sub.borrow().err();
        return false
    }
}



struct ValueFilterContains {
    sub: Rc<RefCell<dyn Index>>,
    filter: Rc<dyn ValueFilterFunction>,
    qs: Rc<RefCell<dyn QuadStore>>,
    result: Option<refs::Ref>,
    err: Option<String>
}

impl ValueFilterContains {
    fn new(qs: Rc<RefCell<dyn QuadStore>>, sub: Rc<RefCell<dyn Index>>, filter: Rc<dyn ValueFilterFunction>) -> Rc<RefCell<ValueFilterContains>> {
        Rc::new(RefCell::new( ValueFilterContains {
            sub, 
            filter,
            qs,
            result: None,
            err: None
        }))
    }

    fn do_filter(&self, val: &refs::Ref) -> bool {
        let qval = self.qs.borrow().name_of(val);
        if qval.is_none() {
            return false
        }
        let res = self.filter.filter(qval.unwrap());
        match res {
            Result::Ok(r) => r,
            Result::Err(_) => false
        }
    }
}

impl fmt::Display for ValueFilterContains {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ValueFilterContains")
    }
}

impl Base for ValueFilterContains {
    fn tag_results(&self, tags: &mut HashMap<String, refs::Ref>) {
        self.sub.borrow().tag_results(tags)
    }

    fn result(&self) -> Option<refs::Ref> {
        self.result.clone()
    }

    fn next_path(&mut self, ctx: &Context) -> bool {
        self.sub.borrow_mut().next_path(ctx)
    }

    fn err(&self) -> Option<String> {
        self.err.clone()
    }

    fn close(&mut self) -> Result<(), String> {
        self.sub.borrow_mut().close()
    }
}

impl Index for ValueFilterContains {
    fn contains(&mut self, ctx: &Context, v:&refs::Ref) -> bool {
        if !self.do_filter(v) {
            return false
        }
        let ok = self.sub.borrow_mut().contains(ctx, v);
        if !ok {
            self.err = self.sub.borrow().err();
        }
        return ok
    }
}