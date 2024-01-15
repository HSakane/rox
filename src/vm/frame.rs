use crate::compiler::object::ClosureObject;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct CallFrame {
    pub closure: Rc<ClosureObject>,
    pub ip: usize,
    pub sp: usize,
}

impl CallFrame {
    pub fn new(closure: Rc<ClosureObject>, ip: usize, sp: usize) -> Self {
        Self { closure, ip, sp }
    }
}
