use super::{Base, Shape, Scanner, Costs, Index, Null, height, is_null, ShapeType};
use super::materialize::Materialize;
use super::super::refs;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use io_context::Context;
use std::fmt;

pub struct And {
    sub: Vec<Rc<RefCell<dyn Shape>>>,
    check_list: Option<Vec<Rc<RefCell<dyn Shape>>>>,
    opt: Option<Vec<Rc<RefCell<dyn Shape>>>>
}


impl And  {

    pub fn new(sub: Vec<Rc<RefCell<dyn Shape>>>) -> Rc<RefCell<And>> {
        Rc::new(RefCell::new(And {
            sub,
            check_list: None,
            opt: None
        }))
    }

    pub fn add_sub_iterator(&mut self, sub: Rc<RefCell<dyn Shape>>) {
        self.sub.push(sub);
    }

    pub fn add_optional_iterator(&mut self, sub: Rc<RefCell<dyn Shape>>) {
        if self.opt.is_none() { 
            self.opt = Some(Vec::new()); 
        }
        self.opt.as_mut().unwrap().push(sub);
    }

    fn optimize_contains(&mut self, ctx: &Context) -> Result<(), String> {
        self.check_list = Some(self.sub.iter().map(|s| s.clone()).collect());
        return sort_by_contains_cost(ctx, self.check_list.as_mut().unwrap())
    }
}

impl fmt::Display for And {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "And")
    }
}

impl Shape for And {
    fn iterate(&self) -> Rc<RefCell<dyn Scanner>> {
        if self.sub.is_empty() {
            return Null::new()
        }

        let mut sub = Vec::new();

        for s in self.sub.iter().skip(1) {
            sub.push(s.borrow().lookup());
        }

        let mut opt = Vec::new();

        if self.opt.is_some() {
            for s in self.opt.as_ref().unwrap() {
                opt.push(s.borrow().lookup());
            }
        }

        AndNext::new(self.sub[0].borrow().iterate(), AndContains::new(sub, opt))
    }

    fn lookup(&self) -> Rc<RefCell<dyn Index>> {
        if self.sub.is_empty() {
            return Null::new()
        }

        let mut sub = Vec::new();
        let check = if self.check_list.is_none() { &self.sub } else { &self.check_list.as_ref().unwrap() };
        
        for s in check {
            sub.push(s.borrow().lookup());
        }

        let mut opt = Vec::new();

        if self.opt.is_some() {
            for s in self.opt.as_ref().unwrap() {
                opt.push(s.borrow().lookup());
            }
        }

        AndContains::new(sub, opt)
    }

    fn stats(&mut self, ctx: &Context) -> Result<Costs, String> {
       let s = get_stats_for_slice(ctx, &self.sub, if self.opt.is_some() { Some(&self.opt.as_ref().unwrap()) } else { None })?;
       Ok(s.0)
    }

    fn optimize(&mut self, ctx: &Context) -> Option<Rc<RefCell<dyn Shape>>>  {

        if self.sub.is_empty() {
            return Some(Null::new())
        }

        let its = optimize_sub_iterators(ctx, &self.sub);

        let out = optimize_replacement(&its);

        if out.is_some() && (self.opt.is_none() || self.opt.as_ref().unwrap().is_empty()) {
            return Some(out.unwrap())
        }

        let its = optimize_order(ctx, &its);

        let its = materialize_its(ctx, &its).unwrap(); // TODO: why is there even an error?

        let new_and = And::new(its);

        if self.opt.is_some() {
            let opt = optimize_sub_iterators(ctx, self.opt.as_ref().unwrap());
            for sub in opt {
                new_and.borrow_mut().add_optional_iterator(sub.clone());
            }
        }

        let _ = new_and.borrow_mut().optimize_contains(ctx);

        // TODO: Logging

        Some(new_and)
    }

    fn sub_iterators(&self) -> Option<Vec<Rc<RefCell<dyn Shape>>>> {
        let mut iters = Vec::new();

        iters.extend(self.sub.iter().map(|s| s.clone()));

        if self.opt.is_some() {
            iters.extend(self.opt.as_ref().unwrap().iter().map(|s| s.clone()));
        }

        Some(iters)
    }

    fn shape_type(&mut self) -> ShapeType {
        ShapeType::And
    }
}

fn optimize_replacement(its: &Vec<Rc<RefCell<dyn Shape>>>) -> Option<Rc<RefCell<dyn Shape>>> {

    if its.is_empty() {
        return Some(Null::new())
    }

    if its.len() == 1 {
        return Some(its[0].clone())
    }

    if has_any_null_iterators(its) {
        println!("optimize_replacement has_any_null_iterators");
        return Some(Null::new())
    }

    None
}

fn optimize_order(ctx: &Context, its: &Vec<Rc<RefCell<dyn Shape>>>) -> Vec<Rc<RefCell<dyn Shape>>> {
    let mut best:Option<&Rc<RefCell<dyn Shape>>> = None;
    let mut best_cost:i64 = 1 << 62;
    let mut best_idx:Option<usize> = None;

    let mut costs = Vec::new();

    for it in its {
        let st = it.borrow_mut().stats(ctx);
        if st.is_ok() {
            costs.push(st.unwrap());
        }
    }

    for (i, root) in its.iter().enumerate() {
        let root_stats = &costs[i];
        let mut cost = root_stats.next_cost;

        for (j, _) in its.iter().enumerate() {
            if i == j { continue; }
            let stats = &costs[j];
            cost += stats.contains_cost * (1 + (root_stats.size.value / (stats.size.value + 1)));
        }
        cost *= root_stats.size.value;
        // TODO logging
        if cost < best_cost {
            best = Some(root);
            best_cost = cost;
            best_idx = Some(i);
        }
    }
    //TODO: logging

    let mut out = Vec::new();

    if best.is_some() {
        out.push(best.unwrap().clone());
    }

    for (i, it) in its.iter().enumerate() {
        if best_idx.is_some() && best_idx.unwrap() == i { continue }
        out.push(it.clone());
    }

    return out;
}

fn sort_by_contains_cost(ctx: &Context, arr:&mut Vec<Rc<RefCell<dyn Shape>>>) -> Result<(), String> {
    // TODO: manage errors better
    // sort arr by cost
    arr.sort_by_cached_key(|s| {
        let res = s.borrow_mut().stats(ctx);
        match res {
            Err(_) => 1 << 62,
            Ok(c) => c.contains_cost
        }
    });

    Ok(())
}

pub fn optimize_sub_iterators(ctx: &Context, its:&Vec<Rc<RefCell<dyn Shape>>>) -> Vec<Rc<RefCell<dyn Shape>>> {
    its.iter().map(|it| {
        let n = it.borrow_mut().optimize(ctx);
        if n.is_some() { n.unwrap() } else { it.clone() }
    }).collect()
}

fn has_any_null_iterators(its:&Vec<Rc<RefCell<dyn Shape>>>) -> bool {
    for it in its {
        if is_null(it) {
            return true
        }
    }
    false
}

fn materialize_its(ctx: &Context, its:&Vec<Rc<RefCell<dyn Shape>>>) -> Result<Vec<Rc<RefCell<dyn Shape>>>, String> {
    let (all_stats, stats) = get_stats_for_slice(ctx, its, None)?;

    let mut out = vec![its[0].clone()];

    for i in 1..its.len() {
        let it = &its[i];
        let st = &stats[i];
        if st.size.value*st.next_cost < (st.contains_cost * (1 + (st.size.value / (all_stats.size.value + 1)))) {
            if height(it, |it| {
                //it.borrow().string() != "Materialize"
                match it.borrow_mut().shape_type() {                                                                 
                    ShapeType::Materialize => false,                                     
                    _ => true,                                                            
                }
            }) > 10 {
                out.push(Materialize::new(it.clone()));
                continue
            }
        }
        out.push(it.clone());
    }

    Ok(out)
}

#[allow(unused)]
fn get_stats_for_slice(ctx: &Context, its:&Vec<Rc<RefCell<dyn Shape>>>, opt:Option<&Vec<Rc<RefCell<dyn Shape>>>>) -> Result<(Costs, Vec::<Costs>), String> {
    if its.is_empty() {
        return Ok((Costs::new(), Vec::new()))
    }

    let mut arr = Vec::new();

    let primary_stats = its[0].borrow_mut().stats(ctx).unwrap();

    let mut contains_cost = primary_stats.contains_cost;
    let mut next_cost = primary_stats.next_cost;
    let mut size = primary_stats.size.value;
    let mut exact = primary_stats.size.exact;

    arr.push(primary_stats.clone());
    
    for i in 1..its.len() {
        let sub = &its[i];
        let stats = sub.borrow_mut().stats(ctx).unwrap();

        next_cost += stats.contains_cost * (1 + (primary_stats.size.value / (stats.size.value + 1)));
        contains_cost += stats.contains_cost;
        if size > stats.size.value {
            size = stats.size.value;
            exact = stats.size.exact;
        }
        arr.push(stats);
    }
    
    Ok((Costs {
        contains_cost: contains_cost,
        next_cost: next_cost,
        size: refs::Size {
            value: size, 
            exact: exact
        }
    }, arr))
}



struct AndNext {
    primary: Rc<RefCell<dyn Scanner>>,
    secondary: Rc<RefCell<dyn Index>>,
    result: Option<refs::Ref>
}


impl AndNext {
    fn new(pri: Rc<RefCell<dyn Scanner>>, sec: Rc<RefCell<dyn Index>>) -> Rc<RefCell<AndNext>> {
        Rc::new(RefCell::new(AndNext {
            primary: pri.clone(),
            secondary: sec.clone(),
            result: None
        }))
    }
}

impl fmt::Display for AndNext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "AndNext")
    }
}

impl Base for AndNext {
    fn tag_results(&self, tags: &mut HashMap<String, refs::Ref>) {
        self.primary.borrow().tag_results(tags);
        self.secondary.borrow().tag_results(tags);
    }

    fn result(&self) -> Option<refs::Ref> {
        if self.result.is_some() {
            Some(self.result.as_ref().unwrap().clone())
        } else {
            None
        }
    }

    fn next_path(&mut self, ctx: &Context) -> bool {
        let mut primary = self.primary.borrow_mut();
        if primary.next_path(ctx) {
            return true
        } else if primary.err().is_some() {
            return false
        }

        let mut secondary = self.secondary.borrow_mut();
        if secondary.next_path(ctx) {
            return true
        } else if secondary.err().is_some() {
            return false
        }

        false
    }

    fn err(&self) -> Option<String> {
        let err = self.primary.borrow().err();
        if err.is_some() {
            return err;
        }
        let err = self.secondary.borrow().err();
        if err.is_some() {
            return err;
        }
        None
    }

    fn close(&mut self) -> Result<(), String> {
        let err = self.primary.borrow_mut().close();
        let err2 = self.secondary.borrow_mut().close();
        
        if let Result::Err(e) = err { 
            Err(e) 
        } else { 
            if let Result::Err(e) = err2 { 
                Err(e) 
            } else { 
                Ok(())
            }
        }
    }
}


impl Scanner for AndNext {
    fn next(&mut self, ctx: &Context) -> bool {
        let mut primary = self.primary.borrow_mut();
        let mut secondary = self.secondary.borrow_mut();
        while primary.next(ctx) {
            let cur = primary.result();

            if cur.is_some() && secondary.contains(ctx, &cur.as_ref().unwrap()) {
                self.result = Some(cur.as_ref().unwrap().clone());
                return true
            }
        }

        false
    }
}


struct AndContains {
    base: Option<Rc<RefCell<dyn Shape>>>,
    sub: Vec<Rc<RefCell<dyn Index>>>,
    opt: Vec<Rc<RefCell<dyn Index>>>,
    opt_check: HashMap<usize, bool>,
    
    result: Option<refs::Ref>,
    err: Option<String>
}


impl AndContains {
    fn new(sub: Vec<Rc<RefCell<dyn Index>>>, opt: Vec<Rc<RefCell<dyn Index>>>) -> Rc<RefCell<AndContains>> {
        Rc::new(RefCell::new(AndContains {
            base: None,
            sub: sub.iter().map(|s| s.clone()).collect(),
            opt: opt.iter().map(|s| s.clone()).collect(),
            opt_check: HashMap::new(),
            result: None,
            err: None
        }))
    }
}


impl fmt::Display for AndContains {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "AndContains")
    }
}


impl Base for AndContains {
    fn tag_results(&self, dst: &mut HashMap<String, refs::Ref>) {
        for sub in &self.sub {
            sub.borrow().tag_results(dst);
        }
        for (i, sub) in self.opt.iter().enumerate() {
            if !self.opt_check.contains_key(&i) {
                continue
            } 
            sub.borrow().tag_results(dst);
        }
    }

    fn result(&self) -> Option<refs::Ref> {
        if self.result.is_some() {
            Some(self.result.as_ref().unwrap().clone())
        } else {
            None
        }
    }

    fn next_path(&mut self, ctx: &Context) -> bool {
        for _sub in &self.sub {
            let mut sub = _sub.borrow_mut();
            if sub.next_path(ctx) {
                return true
            }
            let err = sub.err();
            if err.is_some() {
                self.err = err;
                return false;
            }
        }
        for (i, _sub) in self.opt.iter().enumerate() {
            let mut sub = _sub.borrow_mut();
            if !self.opt_check.contains_key(&i) {
                continue
            }
            if sub.next_path(ctx) {
                return true
            }
            let err = sub.err();
            if err.is_some() {
                self.err = err;
                return false
            }
        }
        false
    }

    fn err(&self) -> Option<String> {
        if self.err.is_some() {
            return self.err.clone()
        }

        for si in &self.sub {
            let err = si.borrow().err();
            if err.is_some() {
                return err
            }
        }

        for si in &self.opt {
            let err = si.borrow().err();
            if err.is_some() {
                return err
            }
        }

        return None
    }

    fn close(&mut self) -> Result<(), String> {
        for sub in &self.sub {
            let res2 = sub.borrow_mut().close();
            if res2.is_err() {
                return res2
            }
        }
        for sub in &self.opt {
            let res2 = sub.borrow_mut().close();
            if res2.is_err()  {
                return res2
            }
        }

        Ok(())
    }
}


impl Index for AndContains {
    fn contains(&mut self, ctx: &Context, val:&refs::Ref) -> bool {
        let prev = self.result.as_ref();
        for (i, _sub) in self.sub.iter().enumerate() {
            let mut sub = _sub.borrow_mut();
            if !sub.contains(ctx, val) {
                let err = sub.err();
                if err.is_some() {
                    self.err = err;
                    return false
                }

                if prev.is_some() {
                    for j in 0..i {
                        let mut sub_j = self.sub[j].borrow_mut();
                        sub_j.contains(ctx, prev.unwrap());
                        let err = sub_j.err();
                        if err.is_some() {
                            self.err = err;
                            return false
                        }
                    }
                }
                
                return false
            }
        }
        self.result = Some(val.clone());
        for (i, sub) in self.opt.iter().enumerate() {
            self.opt_check.insert(i, sub.borrow_mut().contains(ctx, val));
        }
        true
    }
}
