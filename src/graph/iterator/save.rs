use super::{Shape, Base, Index, Scanner, Costs, Tags, ShapeType};
use super::super::refs;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;

pub fn tag<T: ToString>(shape: &Rc<RefCell<dyn Shape>>, tag: &T) -> Rc<RefCell<dyn Shape>> {
    if let ShapeType::Save(save) = shape.borrow_mut().shape_type() {
        save.tags.borrow_mut().add_tags(&vec![tag.to_string()]);
        return shape.clone();
    }
    Save::new(shape.clone(), vec![tag.to_string()])
}

pub struct Save {
    it: Rc<RefCell<dyn Shape>>,
    tags: Rc<RefCell<Tags>>
}

impl Save {
    pub fn new(it: Rc<RefCell<dyn Shape>>, tags: Vec<String>) -> Rc<RefCell<Save>> {
        Rc::new(RefCell::new(Save {
            it,
            tags: Rc::new(RefCell::new(Tags {
                tags: tags.clone(),
                fixed_tags: HashMap::new()
            }))
        }))
    }
}


impl fmt::Display for Save {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Save")
    }
}


impl Shape for Save {
    fn iterate(&self) -> Rc<RefCell<dyn Scanner>> {
        SaveNext::new(self.it.borrow().iterate(), self.tags.clone())
    }

    fn lookup(&self) -> Rc<RefCell<dyn Index>> {
        SaveContains::new(self.it.borrow().lookup(), self.tags.clone())
    }

    fn stats(&mut self) -> Result<Costs, String> {
        self.it.borrow_mut().stats()
    }

    fn optimize(&mut self) -> Option<Rc<RefCell<dyn Shape>>> {
        let res = self.it.borrow_mut().optimize();
        if self.tags.borrow().tags.is_empty() && self.tags.borrow().fixed_tags.is_empty() {
            return res
        }
        if let Some(sub) = res  {
           
            if let ShapeType::Save(save) = sub.borrow_mut().shape_type() {
                save.tags.borrow_mut().copy_from(&self.tags.borrow());
                return Some(sub.clone())
            }

            let s = Save::new(sub, Vec::new());
            s.borrow_mut().tags.borrow_mut().copy_from(&self.tags.borrow());
            Some(s)
        
        } else {
            return None
        }
    }

    fn sub_iterators(&self) -> Option<Vec<Rc<RefCell<dyn Shape>>>> {
        Some(vec![self.it.clone()])
    }


    fn shape_type(&mut self) -> ShapeType {
        ShapeType::Save(self)
    }
}





struct SaveNext {
    it: Rc<RefCell<dyn Scanner>>,
    tags: Rc<RefCell<Tags>>
}

impl SaveNext {
    fn new(it: Rc<RefCell<dyn Scanner>>, tags: Rc<RefCell<Tags>>) -> Rc<RefCell<SaveNext>> {
        Rc::new(RefCell::new(SaveNext {
            it,
            tags
        }))
    }
}

impl fmt::Display for SaveNext {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SaveNext")
    }
}

impl Base for SaveNext {

    fn tag_results(&self, dst: &mut HashMap<String, refs::Ref>) {
        self.it.borrow().tag_results(dst);

        let v = self.result();

        for tag in &self.tags.borrow().tags {
            dst.insert(tag.clone(), v.as_ref().unwrap().clone());
        }

        for (tag, value) in &self.tags.borrow().fixed_tags {
            dst.insert(tag.clone(), value.clone());
        }
    }

    fn result(&self) -> Option<refs::Ref> {
        self.it.borrow().result()
    }

    fn next_path(&mut self) -> bool {
        self.it.borrow_mut().next_path()
    }

    fn err(&self) -> Option<String> {
        self.it.borrow().err()
    }

    fn close(&mut self) -> Result<(), String> {
        self.it.borrow_mut().close()
    }
}

impl Scanner for SaveNext {
    fn next(&mut self) -> bool {
        self.it.borrow_mut().next()
    }
}



struct SaveContains {
    it: Rc<RefCell<dyn Index>>,
    tags: Rc<RefCell<Tags>>
}

impl SaveContains {
    fn new(it: Rc<RefCell<dyn Index>>, tags: Rc<RefCell<Tags>>) -> Rc<RefCell<SaveContains>> {
        Rc::new(RefCell::new(SaveContains {
           it,
           tags
       }))
    }
}

impl fmt::Display for SaveContains {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SaveContains")
    }
}

impl Base for SaveContains {

    fn tag_results(&self, dst: &mut HashMap<String, refs::Ref>) {
        self.it.borrow().tag_results(dst);

        let v = self.result();
        for tag in &self.tags.borrow().tags {
            dst.insert(tag.clone(), v.as_ref().unwrap().clone());
        }

        for (tag, value) in &self.tags.borrow().fixed_tags {
            dst.insert(tag.clone(), value.clone());
        }
    }

    fn result(&self) -> Option<refs::Ref> {
        return self.it.borrow().result()
    }

    fn next_path(&mut self) -> bool {
        return self.it.borrow_mut().next_path()
    }

    fn err(&self) -> Option<String> {
        return self.it.borrow().err()
    }

    fn close(&mut self) -> Result<(), String> {
        return self.it.borrow_mut().close()
    }
}

impl Index for SaveContains {
    fn contains(&mut self, v:&refs::Ref) -> bool {
        return self.it.borrow_mut().contains( v)
    }
}