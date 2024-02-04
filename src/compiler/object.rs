use crate::vm::{chunk::Chunk, value::Value};
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

// #[derive(Debug, Clone, PartialEq, PartialOrd)]
// pub enum ObjectType {
//     Function,
//     Closure,
//     UpValue,
//     Class,
// }

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct FunctionObject {
    pub arity: i32,
    pub chunk: Chunk,
    pub name: String,
    pub upvalue_count: usize,
}

impl FunctionObject {
    pub fn new(name: impl Into<String>, arity: i32) -> Self {
        Self {
            arity,
            chunk: Chunk::new(),
            name: name.into(),
            upvalue_count: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum FunctionType {
    Function,
    Script,
    Method,
    Init,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct ClosureObject {
    pub function: Rc<FunctionObject>,
    pub upvalues: Vec<Rc<RefCell<UpvalueObject>>>,
}

impl ClosureObject {
    pub fn new(function: Rc<FunctionObject>) -> Self {
        let cap = function.upvalue_count;
        Self {
            function,
            upvalues: Vec::with_capacity(cap),
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct UpvalueObject {
    pub location: Option<usize>,
    pub closed: Option<Value>,
    pub next: Option<Rc<RefCell<UpvalueObject>>>,
}

impl UpvalueObject {
    pub fn new(location: usize) -> Self {
        Self {
            location: Some(location),
            closed: None,
            next: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct ClassObject {
    pub name: String,
    pub methods: BTreeMap<Rc<String>, Value>,
}

impl ClassObject {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            methods: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct InstanceObject {
    pub class: Rc<RefCell<ClassObject>>,
    pub fields: BTreeMap<Rc<String>, Value>,
}

impl InstanceObject {
    pub fn new(class: Rc<RefCell<ClassObject>>) -> Self {
        Self {
            class,
            fields: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct BoundMethodObject {
    pub reciever: Value,
    pub method: Rc<ClosureObject>,
}

impl BoundMethodObject {
    pub fn new(reciever: Value, method: Rc<ClosureObject>) -> Self {
        Self { reciever, method }
    }
}
