use serde_json::value::Number;
use io_context::Context;
use super::value::Value;
use super::quad::Quad;



#[derive(PartialEq, Debug, Clone)]
pub struct Size {
    pub value: i64,
    pub exact: bool
}

impl Size {
    pub fn new() -> Size {
        Size {
            value: 0,
            exact: true
        }
    }
}


pub trait Namer {
    fn value_of(&self, v: &Value) -> Option<Ref>;
    fn name_of(&self, key: &Ref) -> Option<Value>;
    
    #[allow(unused)]
    fn values_of(&self, ctx: &Context, values: &Vec<Ref>) -> Result<Vec<Value>, String> {
        Ok(values.iter().map(|v| self.name_of(v).unwrap()).collect())
    }

    #[allow(unused)]
    fn refs_of(&self, ctx: &Context, nodes: &Vec<Value>) -> Result<Vec<Ref>, String> {
        nodes.iter().map(|v| {
            match self.value_of(v) { Some(s) => Ok(s), None => Err("Not Found".to_string()) }
        }).collect()
    }
}


pub fn pre_fetched(v: Value) -> Ref {
    Ref {
        k: v.clone(),
        content: Content::Value(v),
    }
}



#[derive(Debug, PartialEq, Clone)]
pub enum Content {
    None,
    Value(Value),
    Quad(Quad)
}

#[derive(Debug, PartialEq, Clone)]
pub struct Ref {
    pub k: Value,
    pub content: Content
}

impl Ref {
    pub fn none() -> Ref {
        Ref {
            k: Value::None,
            content: Content::None
        }
    }

    // a Ref with key Value::None is used to refer to an exsisting quad but the direction is unassigned
    // this is often the case with the label direction
    // using this method helps to ensure we are checking and handling this scenerio properly
    pub fn key(&self) -> Option<&Value> {
        if let Value::None = self.k {
            return None
        }
        return Some(&self.k)
    }

    pub fn new_i64_node(v: i64) -> Ref {
        let value = Value::Number(Number::from(v));
        Ref {
            k: value.clone(),
            content: Content::Value(value),
        }
    }

    pub fn unwrap_value(&self) -> &Value {
        match &self.content {
            Content::Value(v) => v,
            _ => panic!("Ref does not contain a value")
        }
    }

    pub fn unwrap_quad(&self) -> &Quad {
        match &self.content {
            Content::Quad(q) => q,
            _ => panic!("Ref does not contain a value")
        }
    }

    pub fn has_value(&self) -> bool {
        if let Content::Value(_) = self.content {
            return true
        }
        false
    }

    pub fn has_quad(&self) -> bool {
        if let Content::Quad(_) = self.content {
            return true
        }
        false
    }
}


// impl PartialEq<Value> for Content {
//     fn eq(&self, other: &Value) -> bool {
//         match self {
//             Content::Value(v) => other.eq(v),
//             _ => false
//         }
//     }
// }


// impl PartialEq<Quad> for Content {
//     fn eq(&self, other: &Quad) -> bool {
//         match self {
//             Content::Quad(q) => other.eq(q),
//             _ => false
//         }
//     }
// }


impl PartialEq<Option<Value>> for Content {
    fn eq(&self, other: &Option<Value>) -> bool {
        match self {
            Content::None => other.is_none(),
            Content::Value(v) => other.is_some() && other.as_ref().unwrap().eq(v),
            _ => false
        }
    }
}


impl PartialEq<Option<Quad>> for Content {
    fn eq(&self, other: &Option<Quad>) -> bool {
        match self {
            Content::None => other.is_none(),
            Content::Quad(q) => other.is_some() && other.as_ref().unwrap().eq(q),
            _ => false
        }
    }
}

