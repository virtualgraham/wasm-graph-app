use std::hash::{Hash, Hasher};
use ordered_float::OrderedFloat;
use serde_json::value::Number;

use std::borrow::Cow;
use std::fmt;

// use wasm_bindgen::JsValue;


pub enum Values {
    None,
    Some(Vec<Value>)
}

impl Values {
    pub fn to_vec(self) -> Vec<Value> {
        match self {
            Values::None => Vec::new(),
            Values::Some(v) => v
        }
    }
}

impl From<Option<Value>> for Values {
    fn from(v: Option<Value>) -> Self {
        match v {
            Some(v) => Values::Some(vec![v]),
            None => Values::None
        }
    }
}

impl From<Value> for Values {
    fn from(v: Value) -> Self {
        Values::Some(vec![v])
    }
}

impl From<Vec<Value>> for Values {
    fn from(v: Vec<Value>) -> Self {
        Values::Some(v)
    }
}

impl From<&str> for Values {
    fn from(v: &str) -> Self {
        Values::Some(vec![v.into()])
    }
}

impl From<String> for Values {
    fn from(v: String) -> Self {
        Values::Some(vec![v.into()])
    }
}



#[derive(Debug, PartialEq, Clone)]
#[derive(Serialize, Deserialize)]
pub enum Value {
    Undefined,
    Null,
    Bool(bool),
    Number(Number),
    String(String),
}


impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Undefined => write!(f, "undefined"),
            Value::Null => write!(f, "null"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
        }
        
    }
}


// TODO: implement better hash and partialeq for Value

impl Eq for Value {}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if let Value::Undefined = self {
            "Value::Undefined".hash(state);
        } else if let Value::Null = self {
            "Value::Null".hash(state);
        } else if let Value::Bool(b) = self {
            "Value::Bool".hash(state);
            b.hash(state);
        } else if let Value::Number(n) = self {
            if n.is_f64() {
                "Value::Number::f64".hash(state);
                OrderedFloat::from(n.as_f64().unwrap()).hash(state);
            } else if n.is_i64() {
                "Value::Number::i64".hash(state);
                n.as_i64().hash(state);
            } else if n.is_u64() {
                "Value::Number::u64".hash(state);
                n.as_u64().hash(state);
            }
        } else if let Value::String(s) = self {
            "Value::String".hash(state);
            s.hash(state);
        }
    }
}



macro_rules! from_integer {
    ($($ty:ident)*) => {
        $(
            impl From<$ty> for Value {
                fn from(n: $ty) -> Self {
                    Value::Number(n.into())
                }
            }
        )*
    };
}

from_integer! {
    i8 i16 i32 i64 isize
    u8 u16 u32 u64 usize
}

#[cfg(feature = "arbitrary_precision")]
serde_if_integer128! {
    from_integer! {
        i128 u128
    }
}

impl From<f32> for Value {
    /// Convert 32-bit floating point number to `Value`
    fn from(f: f32) -> Self {
        From::from(f as f64)
    }
}

impl From<f64> for Value {
    /// Convert 64-bit floating point number to `Value`
    fn from(f: f64) -> Self {
        Number::from_f64(f).map_or(Value::Null, Value::Number)
    }
}

impl From<bool> for Value {
    /// Convert boolean to `Value`
    fn from(f: bool) -> Self {
        Value::Bool(f)
    }
}

impl From<String> for Value {
    /// Convert `String` to `Value`
    fn from(f: String) -> Self {
        Value::String(f)
    }
}

impl<'a> From<&'a str> for Value {
    /// Convert string slice to `Value`
    fn from(f: &str) -> Self {
        Value::String(f.to_string())
    }
}

impl<'a> From<Cow<'a, str>> for Value {
    /// Convert copy-on-write string to `Value`
    fn from(f: Cow<'a, str>) -> Self {
        Value::String(f.into_owned())
    }
}

// impl From<&JsValue> for Value {

//     fn from(f: &JsValue) -> Self {
//         if f.is_undefined() {
//             return Value::Undefined
//         } 

//         if f.is_null() {
//             return Value::Null
//         } 
        
//         if f.is_string() {
//             return match f.as_string() {
//                 Some(s) => Value::String(s),
//                 None => Value::Undefined
//             }
//         }

//         match f.as_f64() {
//             Some(n) => Value::from(n),
//             None => match f.as_bool() {
//                 Some(b) => Value::Bool(b),
//                 None => Value::Undefined
//             }
//         }
//     }

// }