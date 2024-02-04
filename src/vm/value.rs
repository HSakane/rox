use crate::compiler::object::{
    BoundMethodObject, ClassObject, ClosureObject, FunctionObject, InstanceObject,
};
use chrono::{DateTime, Local as LocalTime};
use std::{
    cell::RefCell,
    fmt::Display,
    ops::{Add, Div, Mul, Neg, Not, Rem, Sub},
    rc::Rc,
};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Value {
    Float(f64),
    Integer(i64),
    String(Rc<String>),
    Boolean(bool),
    Array(Rc<RefCell<Vec<Value>>>),
    Function(Rc<FunctionObject>),
    Closure(Rc<ClosureObject>),
    Native {
        function: fn(args: &[Value]) -> Value,
    },
    DateTime(DateTime<LocalTime>),
    Class(Rc<RefCell<ClassObject>>),
    Instance(Rc<RefCell<InstanceObject>>),
    BoundMethod(Rc<RefCell<BoundMethodObject>>),
    Null,
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Float(value) => write!(f, "{}", value),
            Value::Integer(value) => write!(f, "{}", value),
            Value::String(value) => write!(f, "{}", value),
            Value::Boolean(value) => write!(f, "{}", value),
            Value::Array(value) => write!(
                f,
                "[{}]",
                value
                    .borrow()
                    .iter()
                    .map(|v| format!("{}", v))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Value::Function(value) => {
                write!(f, "function {} param_len={}", value.name, value.arity)
            }
            Value::Closure(value) => {
                write!(
                    f,
                    "closure {} param_len={} upvalue_len={}",
                    value.function.name,
                    value.function.arity,
                    value.upvalues.len()
                )
            }
            Value::Native { function } => write!(f, "native function {:?}", function),
            Value::DateTime(value) => write!(f, "{}", value.format("%Y/%m/%d %H:%M:%S.%6f")),
            Value::Class(value) => write!(f, "class {}", value.borrow().name),
            Value::Instance(value) => write!(
                f,
                "instance <- class {}",
                &value.borrow().class.borrow().name
            ),
            Value::BoundMethod(value) => write!(
                f,
                "method reciever: {}, name: {}",
                &value.borrow().reciever,
                &value.borrow().method.function.name
            ),
            Value::Null => write!(f, "null"),
        }
    }
}

#[derive(Debug)]
pub enum CalcError {
    Invalid(String),
}

impl Display for CalcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CalcError::Invalid(msg) => write!(f, "{}", msg),
        }
    }
}

impl Value {
    pub fn is_falsy(&self) -> bool {
        match self {
            Value::Boolean(a) => {
                if *a {
                    false
                } else {
                    true
                }
            }
            Value::Null => true,
            _ => false,
        }
    }
}

impl Neg for Value {
    type Output = Result<Self, CalcError>;

    fn neg(self) -> Self::Output {
        match self {
            Value::Float(a) => Ok(Value::Float(-a)),
            Value::Integer(a) => Ok(Value::Integer(-a)),
            other @ _ => Err(CalcError::Invalid(format!("-{}", other))),
        }
    }
}

impl<'a> Neg for &'a Value {
    type Output = Result<Value, CalcError>;

    fn neg(self) -> Self::Output {
        match self {
            Value::Float(a) => Ok(Value::Float(-a)),
            Value::Integer(a) => Ok(Value::Integer(-a)),
            other @ _ => Err(CalcError::Invalid(format!("-{}", other))),
        }
    }
}

impl Not for Value {
    type Output = Result<Self, CalcError>;

    fn not(self) -> Self::Output {
        Ok(Value::Boolean(self.is_falsy()))
    }
}

impl<'a> Not for &'a Value {
    type Output = Result<Value, CalcError>;

    fn not(self) -> Self::Output {
        Ok(Value::Boolean(self.is_falsy()))
    }
}

impl Add for Value {
    type Output = Result<Self, CalcError>;

    fn add(self, rhs: Self) -> Self::Output {
        match self {
            Value::Float(a) => match rhs {
                Value::Float(b) => Ok(Value::Float(a + b)),
                Value::Integer(b) => Ok(Value::Float(a + (b as f64))),
                Value::String(b) => Ok(Value::String(Rc::new(format!("{}{}", a, b)))),
                other @ _ => Err(CalcError::Invalid(format!("{} + {}", self, other))),
            },
            Value::Integer(a) => match rhs {
                Value::Float(b) => Ok(Value::Float((a as f64) + b)),
                Value::Integer(b) => Ok(Value::Integer(a + b)),
                Value::String(b) => Ok(Value::String(Rc::new(format!("{}{}", a, b)))),
                other @ _ => Err(CalcError::Invalid(format!("{} + {}", self, other))),
            },
            Value::String(a) => match rhs {
                Value::Float(b) => Ok(Value::String(Rc::new(format!("{}{}", a, b)))),
                Value::Integer(b) => Ok(Value::String(Rc::new(format!("{}{}", a, b)))),
                Value::String(b) => Ok(Value::String(Rc::new(format!("{}{}", a, b)))),
                Value::Boolean(b) => Ok(Value::String(Rc::new(format!("{}{}", a, b)))),
                b @ Value::Array(_) => Ok(Value::String(Rc::new(format!("{}{}", a, b)))),
                other @ _ => Err(CalcError::Invalid(format!(
                    "{} + {}",
                    Value::String(a),
                    other
                ))),
            },
            Value::Boolean(a) => match rhs {
                Value::String(b) => Ok(Value::String(Rc::new(format!("{}{}", a, b)))),
                other @ _ => Err(CalcError::Invalid(format!("{} + {}", self, other))),
            },
            a @ _ => Err(CalcError::Invalid(format!("{} + {}", a, rhs))),
        }
    }
}

impl<'a> Add for &'a Value {
    type Output = Result<Value, CalcError>;

    fn add(self, rhs: Self) -> Self::Output {
        match self {
            Value::Float(a) => match rhs {
                Value::Float(b) => Ok(Value::Float(a + b)),
                Value::Integer(b) => Ok(Value::Float(a + (*b as f64))),
                Value::String(b) => Ok(Value::String(Rc::new(format!("{}{}", a, b)))),
                other @ _ => Err(CalcError::Invalid(format!("{} + {}", self, other))),
            },
            Value::Integer(a) => match rhs {
                Value::Float(b) => Ok(Value::Float((*a as f64) + b)),
                Value::Integer(b) => Ok(Value::Integer(a + b)),
                Value::String(b) => Ok(Value::String(Rc::new(format!("{}{}", a, b)))),
                other @ _ => Err(CalcError::Invalid(format!("{} + {}", self, other))),
            },
            Value::String(a) => match rhs {
                Value::Float(b) => Ok(Value::String(Rc::new(format!("{}{}", a, b)))),
                Value::Integer(b) => Ok(Value::String(Rc::new(format!("{}{}", a, b)))),
                Value::String(b) => Ok(Value::String(Rc::new(format!("{}{}", a, b)))),
                Value::Boolean(b) => Ok(Value::String(Rc::new(format!("{}{}", a, b)))),
                b @ Value::Array(_) => Ok(Value::String(Rc::new(format!("{}{}", a, b)))),
                other @ _ => Err(CalcError::Invalid(format!(
                    "{} + {}",
                    Value::String(a.clone()),
                    other
                ))),
            },
            Value::Boolean(a) => match rhs {
                Value::String(b) => Ok(Value::String(Rc::new(format!("{}{}", a, b)))),
                other @ _ => Err(CalcError::Invalid(format!("{} + {}", self, other))),
            },
            a @ _ => Err(CalcError::Invalid(format!("{} + {}", a, rhs))),
        }
    }
}

impl Sub for Value {
    type Output = Result<Self, CalcError>;

    fn sub(self, rhs: Self) -> Self::Output {
        match self {
            Value::Float(a) => match rhs {
                Value::Float(b) => Ok(Value::Float(a - b)),
                Value::Integer(b) => Ok(Value::Float(a - (b as f64))),
                other @ _ => Err(CalcError::Invalid(format!("{} - {}", self, other))),
            },
            Value::Integer(a) => match rhs {
                Value::Float(b) => Ok(Value::Float((a as f64) - b)),
                Value::Integer(b) => Ok(Value::Integer(a - b)),
                other @ _ => Err(CalcError::Invalid(format!("{} - {}", self, other))),
            },
            other @ _ => Err(CalcError::Invalid(format!("{} - {}", other, rhs))),
        }
    }
}

impl<'a> Sub for &'a Value {
    type Output = Result<Value, CalcError>;

    fn sub(self, rhs: Self) -> Self::Output {
        match self {
            Value::Float(a) => match rhs {
                Value::Float(b) => Ok(Value::Float(a - b)),
                Value::Integer(b) => Ok(Value::Float(a - (*b as f64))),
                other @ _ => Err(CalcError::Invalid(format!("{} - {}", self, other))),
            },
            Value::Integer(a) => match rhs {
                Value::Float(b) => Ok(Value::Float((*a as f64) - b)),
                Value::Integer(b) => Ok(Value::Integer(a - b)),
                other @ _ => Err(CalcError::Invalid(format!("{} - {}", self, other))),
            },
            other @ _ => Err(CalcError::Invalid(format!("{} - {}", other, rhs))),
        }
    }
}

impl Mul for Value {
    type Output = Result<Self, CalcError>;

    fn mul(self, rhs: Self) -> Self::Output {
        match self {
            Value::Float(a) => match rhs {
                Value::Float(b) => Ok(Value::Float(a * b)),
                Value::Integer(b) => Ok(Value::Float(a * (b as f64))),
                other @ _ => Err(CalcError::Invalid(format!("{} - {}", self, other))),
            },
            Value::Integer(a) => match rhs {
                Value::Float(b) => Ok(Value::Float((a as f64) * b)),
                Value::Integer(b) => Ok(Value::Integer(a * b)),
                other @ _ => Err(CalcError::Invalid(format!("{} - {}", self, other))),
            },
            other @ _ => Err(CalcError::Invalid(format!("{} - {}", other, rhs))),
        }
    }
}

impl<'a> Mul for &'a Value {
    type Output = Result<Value, CalcError>;

    fn mul(self, rhs: Self) -> Self::Output {
        match self {
            Value::Float(a) => match rhs {
                Value::Float(b) => Ok(Value::Float(a * b)),
                Value::Integer(b) => Ok(Value::Float(a * (*b as f64))),
                other @ _ => Err(CalcError::Invalid(format!("{} - {}", self, other))),
            },
            Value::Integer(a) => match rhs {
                Value::Float(b) => Ok(Value::Float((*a as f64) * b)),
                Value::Integer(b) => Ok(Value::Integer(a * b)),
                other @ _ => Err(CalcError::Invalid(format!("{} - {}", self, other))),
            },
            other @ _ => Err(CalcError::Invalid(format!("{} - {}", other, rhs))),
        }
    }
}

impl Div for Value {
    type Output = Result<Self, CalcError>;

    fn div(self, rhs: Self) -> Self::Output {
        match self {
            Value::Float(a) => match rhs {
                Value::Float(b) => Ok(Value::Float(a / b)),
                Value::Integer(b) => Ok(Value::Float(a / (b as f64))),
                other @ _ => Err(CalcError::Invalid(format!("{} - {}", self, other))),
            },
            Value::Integer(a) => match rhs {
                Value::Float(b) => Ok(Value::Float((a as f64) / b)),
                Value::Integer(b) => Ok(Value::Integer(a / b)),
                other @ _ => Err(CalcError::Invalid(format!("{} - {}", self, other))),
            },
            other @ _ => Err(CalcError::Invalid(format!("{} - {}", other, rhs))),
        }
    }
}

impl<'a> Div for &'a Value {
    type Output = Result<Value, CalcError>;

    fn div(self, rhs: Self) -> Self::Output {
        match self {
            Value::Float(a) => match rhs {
                Value::Float(b) => Ok(Value::Float(a / b)),
                Value::Integer(b) => Ok(Value::Float(a / (*b as f64))),
                other @ _ => Err(CalcError::Invalid(format!("{} - {}", self, other))),
            },
            Value::Integer(a) => match rhs {
                Value::Float(b) => Ok(Value::Float((*a as f64) / b)),
                Value::Integer(b) => Ok(Value::Integer(a / b)),
                other @ _ => Err(CalcError::Invalid(format!("{} - {}", self, other))),
            },
            other @ _ => Err(CalcError::Invalid(format!("{} - {}", other, rhs))),
        }
    }
}

impl Rem for Value {
    type Output = Result<Self, CalcError>;

    fn rem(self, rhs: Self) -> Self::Output {
        match self {
            Value::Float(a) => match rhs {
                Value::Float(b) => Ok(Value::Float(a % b)),
                Value::Integer(b) => Ok(Value::Float(a % (b as f64))),
                other @ _ => Err(CalcError::Invalid(format!("{} - {}", self, other))),
            },
            Value::Integer(a) => match rhs {
                Value::Float(b) => Ok(Value::Float((a as f64) % b)),
                Value::Integer(b) => Ok(Value::Integer(a % b)),
                other @ _ => Err(CalcError::Invalid(format!("{} - {}", self, other))),
            },
            other @ _ => Err(CalcError::Invalid(format!("{} - {}", other, rhs))),
        }
    }
}

impl<'a> Rem for &'a Value {
    type Output = Result<Value, CalcError>;

    fn rem(self, rhs: Self) -> Self::Output {
        match self {
            Value::Float(a) => match rhs {
                Value::Float(b) => Ok(Value::Float(a % b)),
                Value::Integer(b) => Ok(Value::Float(a % (*b as f64))),
                other @ _ => Err(CalcError::Invalid(format!("{} - {}", self, other))),
            },
            Value::Integer(a) => match rhs {
                Value::Float(b) => Ok(Value::Float((*a as f64) % b)),
                Value::Integer(b) => Ok(Value::Integer(a % b)),
                other @ _ => Err(CalcError::Invalid(format!("{} - {}", self, other))),
            },
            other @ _ => Err(CalcError::Invalid(format!("{} - {}", other, rhs))),
        }
    }
}

const CONSTANT_LEN: usize = 256;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct ConstantArray {
    values: [Value; CONSTANT_LEN],
    index: usize,
}

impl ConstantArray {
    pub fn new() -> Self {
        Self {
            values: vec![Value::Null; CONSTANT_LEN].try_into().unwrap(),
            index: 0,
        }
    }

    pub fn push(&mut self, value: Value) {
        self.values[self.index] = value;
        self.index += 1;
    }

    pub fn get(&self, index: usize) -> Value {
        unsafe { self.values.get_unchecked(index).clone() }
    }

    pub fn len(&self) -> usize {
        self.index
    }
}

const STACK_LEN: usize = 256;

#[derive(Debug, Clone)]
pub struct StackArray {
    values: [Value; STACK_LEN],
    index: usize,
}

impl StackArray {
    pub fn new() -> Self {
        Self {
            values: vec![Value::Null; STACK_LEN].try_into().unwrap(),
            index: 0,
        }
    }

    pub fn push(&mut self, value: Value) {
        self.values[self.index] = value;
        self.index += 1;
    }

    pub fn pop(&mut self) -> Value {
        self.index -= 1;
        unsafe { self.values.get_unchecked(self.index).clone() }
    }

    pub fn pop_index(&mut self) {
        self.index -= 1;
    }

    pub fn last(&self) -> &Value {
        let index = self.index - 1;
        unsafe { self.values.get_unchecked(index) }
    }

    pub fn set_index(&mut self, index: usize) {
        self.index = index;
    }

    pub fn get(&self, index: usize) -> &Value {
        unsafe { self.values.get_unchecked(index) }
    }

    pub fn get_mut(&mut self, index: usize) -> &mut Value {
        unsafe { self.values.get_unchecked_mut(index) }
    }

    pub fn len(&self) -> usize {
        self.index
    }

    pub fn get_slice(&self, offset: usize) -> &[Value] {
        &self.values[offset..self.index]
    }

    pub fn print(&self) {
        let stack = self.values[0..self.index]
            .iter()
            .map(|f| format!("[{}]", f))
            .collect::<Vec<_>>()
            .join(" - ");
        println!("[index]{} [values]{}", self.index, stack);
    }
}
