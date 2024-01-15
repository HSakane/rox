use super::value::Value;
use rustc_hash::FxHasher;
use std::{collections::HashMap, fmt::Display, hash::BuildHasherDefault, rc::Rc};

type Hasher = BuildHasherDefault<FxHasher>;

#[derive(Debug, Clone, PartialEq)]
pub struct Table {
    identifer: HashMap<Rc<String>, Value, Hasher>,
}

impl Display for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut result = String::new();
        result.push_str("[identifer]\n");
        for (k, v) in self.identifer.iter() {
            result.push_str(format!("{}={}\n", k, v).as_str())
        }
        write!(f, "{}", result)
    }
}

impl Table {
    pub fn new() -> Self {
        Self {
            identifer: HashMap::default(),
        }
    }

    pub fn insert(&mut self, key: Rc<String>, value: Value) -> Option<Value> {
        self.identifer.insert(key, value)
    }

    pub fn find(&mut self, key: &String) -> Option<&Value> {
        self.identifer.get(key)
    }
}
