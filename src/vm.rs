use self::{
    chunk::{
        OP_ADD, OP_ARRAY, OP_CALL, OP_CLASS, OP_CLOSE_UPVALUE, OP_CLOSURE, OP_CONSTANT, OP_COUNTUP,
        OP_DEFINE_GLOBAL, OP_DIVIDE, OP_EQUAL, OP_FALSE, OP_GET_GLOBAL, OP_GET_LOCAL, OP_GET_PROP,
        OP_GET_SUPER, OP_GET_UPVALUE, OP_GREATER, OP_INDEX_CALL, OP_INDEX_SET, OP_INHERIT,
        OP_CONSTANT0, OP_INVOKE, OP_JUMP, OP_JUMP_IF_FALSE, OP_JUMP_IF_RANGE_END, OP_LESS,
        OP_LOOP, OP_METHOD, OP_MULTIPLY, OP_NEGATIVE, OP_NOT, OP_NULL, OP_POP, OP_POW, OP_PRINT,
        OP_RANGE, OP_REM, OP_RETURN, OP_SET_GLOBAL, OP_SET_LOCAL, OP_SET_PROP, OP_SET_UPVALUE,
        OP_SUBTRACT, OP_SUPER_INVOKE, OP_TRUE,
    },
    frame::CallFrame,
    table::Table,
    value::{StackArray, Value},
};
use crate::compiler::object::{
    BoundMethodObject, ClassObject, ClosureObject, FunctionObject, InstanceObject, UpvalueObject,
};
use chrono::Local as LocalTime;
use std::{cell::RefCell, rc::Rc};

pub mod chunk;
pub mod frame;
pub mod table;
pub mod value;

pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError(String),
    End,
}

const FRAME_MAX: usize = 256;

pub struct VM {
    stack: StackArray,
    globals: Table,
    frames: [CallFrame; FRAME_MAX],
    frame_count: usize,
    open_upvalue: Option<Rc<RefCell<UpvalueObject>>>,
}

impl VM {
    pub fn new(frame: CallFrame) -> Self {
        let mut vm = VM {
            stack: StackArray::new(),
            globals: Table::new(),
            frames: vec![
                CallFrame::new(
                    Rc::new(ClosureObject::new(Rc::new(FunctionObject::new("", 0)))),
                    0,
                    0
                );
                FRAME_MAX
            ]
            .try_into()
            .unwrap(),
            frame_count: 0,
            open_upvalue: None,
        };
        vm.stack.push(Value::Closure(frame.closure.clone()));
        Self::frame_push(&mut vm, frame);
        vm
    }

    pub fn interpret(&mut self) -> InterpretResult {
        self.register_native();
        return self.run();
    }

    fn register_native(&mut self) {
        self.globals
            .insert(Rc::new("len".to_string()), Value::Native { function: len });
        self.globals.insert(
            Rc::new("append".to_string()),
            Value::Native { function: append },
        );
        self.globals.insert(
            Rc::new("last".to_string()),
            Value::Native { function: last },
        );
        self.globals.insert(
            Rc::new("rest".to_string()),
            Value::Native { function: rest },
        );
        self.globals
            .insert(Rc::new("str".to_string()), Value::Native { function: str });
        self.globals.insert(
            Rc::new("first".to_string()),
            Value::Native { function: first },
        );
        self.globals
            .insert(Rc::new("now".to_string()), Value::Native { function: now });
        self.globals.insert(
            Rc::new("range".to_string()),
            Value::Native { function: range },
        );
        self.globals
            .insert(Rc::new("get".to_string()), Value::Native { function: get });
    }

    fn run(&mut self) -> InterpretResult {
        loop {
            let instruction = match self
                .frame_last()
                .closure
                .function
                .chunk
                .get_instruction(self.get_current_ip())
            {
                Some(i) => *i,
                None => {
                    return InterpretResult::End;
                }
            };
            // self.frame_last().closure.function.chunk.disassemble_instruction(self.get_current_ip());
            *self.get_current_ip_mut() += 1;

            match instruction {
                OP_RETURN => {
                    let result = self.stack.pop();
                    self.close_upvalues();

                    let frame = self.frame_pop();
                    if frame.closure.function.name == "__main__" {
                        self.stack.pop_index();
                        return InterpretResult::Ok;
                    }
                    let index = frame.sp;
                    self.stack.set_index(index);
                    self.stack.push(result);
                }
                OP_CONSTANT => {
                    let value = Self::read_constant(&mut self.frame_last_mut());
                    self.stack.push(value);
                }
                OP_NEGATIVE => {
                    let a = self.stack.pop();
                    match -a {
                        Ok(value) => self.stack.push(value),
                        Err(error) => {
                            return InterpretResult::RuntimeError(format!(
                                "Instruction is \"OP_NEGATIVE\". [Not Support Operation]{}",
                                error
                            ))
                        }
                    }
                }
                OP_ADD => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    match a + b {
                        Ok(value) => self.stack.push(value),
                        Err(error) => {
                            return InterpretResult::RuntimeError(format!(
                                "Instruction is \"OP_ADD\". [Not Support Operation]{}",
                                error
                            ))
                        }
                    }
                }
                OP_SUBTRACT => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    match a - b {
                        Ok(value) => self.stack.push(value),
                        Err(error) => {
                            return InterpretResult::RuntimeError(format!(
                                "Instruction is \"OP_SUBTRACT\". [Not Support Operation]{}",
                                error
                            ))
                        }
                    }
                }
                OP_MULTIPLY => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    match a * b {
                        Ok(value) => self.stack.push(value),
                        Err(error) => {
                            return InterpretResult::RuntimeError(format!(
                                "Instruction is \"OP_MULTIPLY\". [Not Support Operation]{}",
                                error
                            ))
                        }
                    }
                }
                OP_DIVIDE => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    match a / b {
                        Ok(value) => self.stack.push(value),
                        Err(error) => {
                            return InterpretResult::RuntimeError(format!(
                                "Instruction is \"OP_DIVIDE\". [Not Support Operation]{}",
                                error
                            ))
                        }
                    }
                }
                OP_REM => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    match a % b {
                        Ok(value) => self.stack.push(value),
                        Err(error) => {
                            return InterpretResult::RuntimeError(format!(
                                "Instruction is \"OP_DIVIDE\". [Not Support Operation]{}",
                                error
                            ))
                        }
                    }
                }
                OP_POW => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    let value = match a {
                        Value::Float(a) => match b {
                            Value::Float(b) => Value::Float(a.powf(b)),
                            Value::Integer(b) => Value::Float(a.powi(b as i32)),
                            _ => {
                                return InterpretResult::RuntimeError(format!(
                                    "Instruction is \"OP_POW\". [Not Support Operation]{} ^ {}",
                                    a, b
                                ))
                            }
                        },
                        Value::Integer(a) => match b {
                            Value::Float(b) => Value::Float((a as f64).powf(b)),
                            Value::Integer(b) => Value::Float((a as f64).powi(b as i32)),
                            _ => {
                                return InterpretResult::RuntimeError(format!(
                                    "Instruction is \"OP_POW\". [Not Support Operation]{} ^ {}",
                                    a, b
                                ))
                            }
                        },
                        _ => {
                            return InterpretResult::RuntimeError(format!(
                                "Instruction is \"OP_POW\". [Not Support Operation]{} ^ {}",
                                a, b
                            ))
                        }
                    };
                    self.stack.push(value);
                }
                OP_TRUE => {
                    self.stack.push(Value::Boolean(true));
                }
                OP_FALSE => {
                    self.stack.push(Value::Boolean(false));
                }
                OP_NULL => {
                    self.stack.push(Value::Null);
                }
                OP_NOT => {
                    let a = self.stack.pop();
                    match !a {
                        Ok(value) => self.stack.push(value),
                        Err(error) => {
                            return InterpretResult::RuntimeError(format!(
                                "Instruction is \"OP_NOT\". [Not Support Operation]{}",
                                error
                            ))
                        }
                    }
                }
                OP_GREATER => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    if a > b {
                        self.stack.push(Value::Boolean(true));
                    } else {
                        self.stack.push(Value::Boolean(false));
                    }
                }
                OP_LESS => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    if a < b {
                        self.stack.push(Value::Boolean(true));
                    } else {
                        self.stack.push(Value::Boolean(false));
                    }
                }
                OP_EQUAL => {
                    let b = self.stack.pop();
                    let a = self.stack.pop();
                    if a == b {
                        self.stack.push(Value::Boolean(true));
                    } else {
                        self.stack.push(Value::Boolean(false));
                    }
                }
                OP_PRINT => {
                    let a = self.stack.pop();
                    println!("{}", a);
                }
                OP_POP => {
                    self.stack.pop_index();
                }
                OP_DEFINE_GLOBAL => {
                    let key = Self::read_string(&mut self.frame_last_mut());
                    let value = self.stack.pop();
                    self.globals.insert(key, value);
                }
                OP_GET_GLOBAL => {
                    let key = Self::read_string(&mut self.frame_last_mut());
                    match self.globals.find(&key) {
                        Some(value) => self.stack.push(value.clone()),
                        None => {
                            return InterpretResult::RuntimeError(format!(
                                "Instruction is \"OP_GET_GLOBAL\". not found identifer name.({})",
                                key
                            ))
                        }
                    };
                }
                OP_SET_GLOBAL => {
                    let key = Self::read_string(&mut self.frame_last_mut());
                    let value = self.stack.last().clone();
                    self.globals.insert(key, value);
                }
                OP_GET_LOCAL => {
                    let index = match Self::read_local_index(&mut self.frame_last_mut()) {
                        Some(index) => index,
                        None => {
                            return InterpretResult::RuntimeError(
                                "Instruction is \"OP_GET_LOCAL\". but no value.".to_string(),
                            )
                        }
                    };
                    let sp = self.frame_last().sp;
                    let value = self.stack.get(sp + index).clone();
                    self.stack.push(value);
                }
                OP_SET_LOCAL => {
                    let index = match Self::read_local_index(&mut self.frame_last_mut()) {
                        Some(index) => index,
                        None => {
                            return InterpretResult::RuntimeError(
                                "Instruction is \"OP_SET_LOCAL\". but no value.".to_string(),
                            )
                        }
                    };
                    let value = self.stack.last().clone();
                    let sp = self.frame_last().sp;
                    *self.stack.get_mut(sp + index) = value;
                }
                OP_JUMP_IF_FALSE => {
                    let offset = match Self::read_jump(&mut self.frame_last_mut()) {
                        Some(offset) => offset,
                        None => return InterpretResult::RuntimeError(
                            "Instruction is \"OP_JUMP_IF_FALSE\". but no offset on instruction."
                                .to_string(),
                        ),
                    };
                    if self.stack.last().is_falsy() {
                        *self.get_current_ip_mut() += offset;
                    }
                }
                OP_JUMP => {
                    let offset = match Self::read_jump(&mut self.frame_last_mut()) {
                        Some(offset) => offset,
                        None => {
                            return InterpretResult::RuntimeError(
                                "Instruction is \"OP_JUMP\". but no offset on instruction."
                                    .to_string(),
                            )
                        }
                    };
                    *self.get_current_ip_mut() += offset;
                }
                OP_LOOP => {
                    let offset = match Self::read_jump(&mut self.frame_last_mut()) {
                        Some(offset) => offset,
                        None => {
                            return InterpretResult::RuntimeError(
                                "Instruction is \"OP_LOOP\". but no offset on instruction."
                                    .to_string(),
                            )
                        }
                    };
                    *self.get_current_ip_mut() -= offset;
                }
                OP_CALL => {
                    let arg_count = match Self::read_byte(&mut self.frame_last_mut()) {
                        Some(arg_count) => arg_count as usize,
                        None => {
                            return InterpretResult::RuntimeError(
                                "Instruction is \"OP_CALL\". but no offset on instruction."
                                    .to_string(),
                            )
                        }
                    };

                    let index = self.stack.len() - (arg_count + 1) as usize;
                    let callee = self.stack.get(index).clone();
                    match self.call_value(callee, arg_count) {
                        Ok(_) => {}
                        Err(e) => return InterpretResult::RuntimeError(e),
                    }
                }
                OP_ARRAY => {
                    let length = match Self::read_byte(&mut self.frame_last_mut()) {
                        Some(length) => length as usize,
                        None => {
                            return InterpretResult::RuntimeError(
                                "Instruction is \"OP_ARRAY\". but no offset on instruction."
                                    .to_string(),
                            )
                        }
                    };
                    let values = self
                        .stack
                        .get_slice(self.stack.len() - length)
                        .iter()
                        .map(|v| v.clone())
                        .collect::<Vec<Value>>();
                    self.stack.set_index(self.stack.len() - length);
                    self.stack.push(Value::Array(Rc::new(RefCell::new(values))));
                }
                OP_INDEX_CALL => {
                    let b = match self.stack.pop() {
                        Value::Integer(v) => v as usize,
                        _ => {
                            return InterpretResult::RuntimeError(
                                "Instruction is \"OP_INDEX_CALL\". but no value.".to_string(),
                            )
                        }
                    };
                    let a = match self.stack.pop() {
                        Value::Array(v) => v,
                        _ => {
                            return InterpretResult::RuntimeError(
                                "Instruction is \"OP_INDEX_CALL\". but no value.".to_string(),
                            )
                        }
                    };
                    match a.borrow().get(b) {
                        Some(v) => self.stack.push(v.clone()),
                        None => self.stack.push(Value::Null),
                    };
                }
                OP_CLOSURE => {
                    let value = Self::read_constant(&mut self.frame_last_mut());
                    match value {
                        Value::Function(func) => {
                            let range = 0..func.upvalue_count;
                            let mut closure_object = ClosureObject::new(func);
                            for _ in range {
                                let is_local = match Self::read_byte(&mut self.frame_last_mut()) {
                                    Some(length) => length,
                                    None => {
                                        return InterpretResult::RuntimeError(
                                            "Instruction is \"OP_CLOSURE\". but no offset on instruction."
                                                .to_string(),
                                        )
                                    }
                                };
                                let upvalue_index = match Self::read_byte(&mut self.frame_last_mut()) {
                                    Some(length) => length as usize,
                                    None => {
                                        return InterpretResult::RuntimeError(
                                            "Instruction is \"OP_CLOSURE\". but no offset on instruction."
                                                .to_string(),
                                        )
                                    }
                                };
                                if is_local == 1 {
                                    closure_object.upvalues.push(
                                        self.capture_upvalue(self.frame_last().sp + upvalue_index),
                                    )
                                } else {
                                    closure_object.upvalues.push(
                                        self.frame_last().closure.upvalues[upvalue_index].clone(),
                                    )
                                }
                            }
                            self.stack.push(Value::Closure(Rc::new(closure_object)));
                        }
                        _ => todo!(),
                    }
                }
                OP_CLOSE_UPVALUE => {
                    self.close_upvalues();
                    self.stack.pop();
                }
                OP_GET_UPVALUE => {
                    let upvalue_index = match Self::read_byte(&mut self.frame_last_mut()) {
                        Some(length) => length as usize,
                        None => {
                            return InterpretResult::RuntimeError(
                                "Instruction is \"OP_CLOSURE\". but no offset on instruction."
                                    .to_string(),
                            )
                        }
                    };
                    let closed_value = match &self.frame_last().closure.upvalues[upvalue_index]
                        .borrow()
                        .closed
                    {
                        Some(val) => val.clone(),
                        None => {
                            match &self.frame_last().closure.upvalues[upvalue_index]
                                .borrow()
                                .location
                            {
                                Some(loc) => self.stack.get(*loc).clone(),
                                None => panic!(""),
                            }
                        }
                    };
                    self.stack.push(closed_value);
                }
                OP_SET_UPVALUE => {
                    let upvalue_index = match Self::read_byte(&mut self.frame_last_mut()) {
                        Some(length) => length as usize,
                        None => {
                            return InterpretResult::RuntimeError(
                                "Instruction is \"OP_CLOSURE\". but no offset on instruction."
                                    .to_string(),
                            )
                        }
                    };
                    self.frame_last().closure.upvalues[upvalue_index]
                        .borrow_mut()
                        .closed = Some(self.stack.last().clone());
                }
                OP_CLASS => {
                    let name = Self::read_string(&mut self.frame_last_mut());
                    let class = Value::Class(Rc::new(RefCell::new(ClassObject::new(&*name))));
                    self.stack.push(class);
                }
                OP_GET_PROP => {
                    let instance = self.stack.last().clone();
                    let name = Self::read_string(&mut self.frame_last_mut());
                    match instance {
                        Value::Instance(instance) => {
                            if let Some(value) = instance.borrow().fields.get(&name) {
                                self.stack.pop_index();
                                self.stack.push(value.clone());
                                continue;
                            }
                            match self.bind_method(instance.borrow().class.clone(), name.clone()) {
                                Some(_) => continue,
                                None => {}
                            }
                            self.stack.pop_index();
                            self.stack.push(Value::Null);
                        }
                        invalid => {
                            self.stack.print();
                            panic!("[OP_GET_PROP]{:?}", invalid);
                        }
                    };
                }
                OP_SET_PROP => {
                    let instance = self.stack.get(self.stack.len() - 2).clone();
                    let name = Self::read_string(&mut self.frame_last_mut());
                    let value = self.stack.last().clone();

                    match instance {
                        Value::Instance(instance) => {
                            instance.borrow_mut().fields.insert(name, value.clone());
                        }
                        invalid => {
                            self.stack.print();
                            panic!("[OP_SET_PROP] invalid. {}", invalid);
                        }
                    };
                    self.stack.pop_index();
                    self.stack.pop_index();
                    self.stack.push(value);
                }
                OP_METHOD => {
                    let name = Self::read_string(&mut self.frame_last_mut());
                    let method = self.stack.last().clone();
                    let class = match self.stack.get(self.stack.len() - 2) {
                        Value::Class(cls) => cls.clone(),
                        _ => todo!(),
                    };
                    class.borrow_mut().methods.insert(name, method);
                    self.stack.pop_index();
                }
                OP_INVOKE => {
                    let name = Self::read_string(&mut self.frame_last_mut());
                    let arg_count = match Self::read_byte(&mut self.frame_last_mut()) {
                        Some(arg_count) => arg_count as usize,
                        None => {
                            return InterpretResult::RuntimeError(
                                "Instruction is \"OP_INVOKE\". but no offset on instruction."
                                    .to_string(),
                            )
                        }
                    };
                    match self.invoke(name, arg_count) {
                        Ok(_) => {}
                        Err(e) => return InterpretResult::RuntimeError(e),
                    };
                }
                OP_INHERIT => {
                    let index = self.stack.len() - 2;
                    let super_class = match self.stack.get(index).clone() {
                        Value::Class(cls) => cls,
                        invalid => return InterpretResult::RuntimeError(format!(
                            "Instruction is \"OP_INHERIT\". Super class must be class. But found {}",
                            invalid
                        )),
                    };
                    let sub_class = match self.stack.last().clone() {
                        Value::Class(cls) => cls,
                        invalid => {
                            return InterpretResult::RuntimeError(format!(
                            "Instruction is \"OP_INHERIT\". Sub class must be class. But found {}",
                            invalid
                        ))
                        }
                    };
                    for (k, v) in super_class.borrow().methods.iter() {
                        sub_class.borrow_mut().methods.insert(k.clone(), v.clone());
                    }
                    self.stack.pop_index();
                }
                OP_SUPER_INVOKE => {
                    let name = Self::read_string(&mut self.frame_last_mut());
                    let arg_count = match Self::read_byte(&mut self.frame_last_mut()) {
                        Some(arg_count) => arg_count as usize,
                        None => {
                            return InterpretResult::RuntimeError(
                                "Instruction is \"OP_INVOKE\". but no offset on instruction."
                                    .to_string(),
                            )
                        }
                    };
                    match self.stack.pop().clone() {
                        Value::Class(cls) => {
                            match self.invoke_from_class(cls, name, arg_count) {
                                Ok(_) => {}
                                Err(e) => return InterpretResult::RuntimeError(e),
                            };
                        }
                        invalid => {
                            return InterpretResult::RuntimeError(format!("invalid: {}", invalid))
                        }
                    };
                }
                OP_GET_SUPER => {
                    let super_class = self.stack.pop().clone();
                    let name = Self::read_string(&mut self.frame_last_mut());
                    match super_class {
                        Value::Class(super_class) => {
                            match self.bind_method(super_class.clone(), name.clone()) {
                                Some(_) => continue,
                                None => {}
                            }
                            self.stack.pop_index();
                            self.stack.push(Value::Null);
                        }
                        invalid => {
                            self.stack.print();
                            panic!("[OP_SUPER_GET_PROP]{:?}", invalid);
                        }
                    };
                }
                OP_INDEX_SET => {
                    let value = self.stack.pop();
                    let index = match self.stack.pop() {
                        Value::Integer(v) => v as usize,
                        _ => {
                            return InterpretResult::RuntimeError(
                                "Instruction is \"OP_INDEX_SET\". but no value.".to_string(),
                            )
                        }
                    };
                    let array = match self.stack.pop() {
                        Value::Array(v) => v,
                        _ => {
                            return InterpretResult::RuntimeError(
                                "Instruction is \"OP_INDEX_SET\". but no value.".to_string(),
                            )
                        }
                    };
                    match array.borrow_mut().get_mut(index) {
                        Some(get_val) => *get_val = value,
                        None => {
                            return InterpretResult::RuntimeError(
                                "Instruction is \"OP_INDEX_SET\". but no value.".to_string(),
                            )
                        }
                    };
                }
                OP_CONSTANT0 => {
                    self.stack.push(Value::Integer(0));
                }
                OP_JUMP_IF_RANGE_END => {
                    let range = self.stack.pop();
                    let index = match self.stack.pop() {
                        Value::Integer(i) => i as usize,
                        invalid => panic!("Range index expected integer. But found {}", invalid),
                    };
                    let offset = match Self::read_jump(&mut self.frame_last_mut()) {
                        Some(offset) => offset,
                        None => return InterpretResult::RuntimeError(
                            "Instruction is \"OP_JUMP_IF_FALSE\". but no offset on instruction."
                                .to_string(),
                        ),
                    };
                    match range {
                        Value::Array(array) => match array.borrow().get(index) {
                            Some(v) => {
                                self.stack.push(v.clone());
                            }
                            None => {
                                self.stack.push(Value::Null);
                                *self.get_current_ip_mut() += offset;
                            }
                        },
                        invalid => panic!("Range expected array. But found {}", invalid),
                    };
                }
                OP_COUNTUP => {
                    let index = match Self::read_local_index(&mut self.frame_last_mut()) {
                        Some(index) => index,
                        None => {
                            return InterpretResult::RuntimeError(
                                "Instruction is \"OP_SET_LOCAL\". but no value.".to_string(),
                            )
                        }
                    };
                    let sp = self.frame_last().sp;
                    let value = self.stack.get(sp + index).clone();
                    *self.stack.get_mut(sp + index) = match value {
                        Value::Integer(i) => Value::Integer(i + 1),
                        invalid => panic!("invalid: {}", invalid),
                    };
                }
                OP_RANGE => {
                    let end = match self.stack.pop() {
                        Value::Integer(i) => i,
                        invalid => panic!("invalid: {}", invalid),
                    };
                    let start = match self.stack.pop() {
                        Value::Integer(i) => i,
                        invalid => panic!("invalid: {}", invalid),
                    };
                    let mut values: Vec<Value> = Vec::new();
                    for i in start..=end {
                        values.push(Value::Integer(i));
                    }
                    self.stack.push(Value::Array(Rc::new(RefCell::new(values))));
                }
                _ => {
                    return InterpretResult::CompileError;
                }
            }
        }
    }

    fn call_value(&mut self, callee: Value, arg_count: usize) -> Result<(), String> {
        let index = self.stack.len() - (arg_count + 1) as usize;
        match &callee {
            Value::Closure(closure) => {
                if arg_count != closure.function.arity as usize {
                    return Err(format!(
                        "expected arg_count eq {} but found {}.",
                        closure.function.arity, arg_count
                    ));
                }

                let frame = CallFrame::new(Rc::clone(closure), 0, index);
                self.frame_push(frame);
            }
            Value::Native { function } => {
                let args = self.stack.get_slice(self.stack.len() - arg_count as usize);
                let value = function(args);
                for _ in 0..arg_count {
                    self.stack.pop_index();
                }
                self.stack.pop_index();
                self.stack.push(value);
            }
            Value::Class(class) => {
                let value =
                    Value::Instance(Rc::new(RefCell::new(InstanceObject::new(class.clone()))));
                let init = match class.borrow().methods.get(&Rc::new("init".to_string())) {
                    Some(v) => Some(v.clone()),
                    None => None,
                };

                let tmp = self.stack.get_mut(index);
                *tmp = value;

                match init {
                    Some(init) => match init {
                        Value::Closure(closure) => {
                            if arg_count != closure.function.arity as usize {
                                return Err(format!(
                                    "expected arg_count eq {} but found {}.",
                                    closure.function.arity, arg_count
                                ));
                            }

                            let frame = CallFrame::new(closure, 0, index);
                            self.frame_push(frame);
                        }
                        invalid => panic!("expected closure but found {}.", invalid),
                    },
                    None => {
                        if arg_count != 0 {
                            panic!("expected arg_count eq 0 but found {}.", arg_count);
                        }
                    }
                }
            }
            Value::BoundMethod(bound) => {
                let closure = bound.borrow().method.clone();
                if arg_count != closure.function.arity as usize {
                    return Err(format!(
                        "expected arg_count eq {} but found {}.",
                        closure.function.arity, arg_count
                    ));
                }

                let reciever = bound.borrow().reciever.clone();
                let tmp = self.stack.get_mut(index);
                *tmp = reciever;
                let frame = CallFrame::new(closure, 0, index);
                self.frame_push(frame);
            }
            other => {
                return Err(format!(
                    "Instruction is \"OP_CALL\". not found identifer name.({})",
                    other
                ))
            }
        }
        Ok(())
    }

    fn invoke(&mut self, name: Rc<String>, arg_count: usize) -> Result<(), String> {
        let index = self.stack.len() - (arg_count + 1) as usize;
        let receiver = self.stack.get(index).clone();
        match receiver {
            Value::Instance(instance) => {
                match instance.borrow().fields.get(&name) {
                    Some(field) => {
                        let val = self.stack.get_mut(index);
                        *val = field.clone();
                        self.call_value(field.clone(), arg_count)?;
                    }
                    None => {
                        self.invoke_from_class(instance.borrow().class.clone(), name, arg_count)?;
                    }
                };
            }
            _ => todo!(),
        };
        Ok(())
    }

    fn invoke_from_class(
        &mut self,
        class: Rc<RefCell<ClassObject>>,
        name: Rc<String>,
        arg_count: usize,
    ) -> Result<(), String> {
        let index = self.stack.len() - (arg_count + 1) as usize;
        let bound_method = match class.borrow().methods.get(&name) {
            Some(bound_method) => bound_method.clone(),
            None => panic!("undefined method {}", name),
        };
        match bound_method {
            Value::Closure(closure) => {
                if arg_count != closure.function.arity as usize {
                    return Err(format!(
                        "expected arg_count eq {} but found {}.",
                        closure.function.arity, arg_count
                    ));
                }
                let frame = CallFrame::new(Rc::clone(&closure), 0, index);
                self.frame_push(frame);
            }
            invalid => return Err(format!("invalid: {:?}", invalid)),
        }
        Ok(())
    }

    fn bind_method(&mut self, class: Rc<RefCell<ClassObject>>, name: Rc<String>) -> Option<()> {
        let method = match class.borrow().methods.get(&name) {
            Some(method) => match method {
                Value::Closure(closure) => closure.clone(),
                _ => return None,
            },
            None => return None,
        };

        let bound_method = BoundMethodObject::new(self.stack.last().clone(), method);
        self.stack.pop();
        self.stack
            .push(Value::BoundMethod(Rc::new(RefCell::new(bound_method))));
        Some(())
    }

    fn print_upvalue(no: usize, upvalue: &UpvalueObject) {
        println!("[No {}]", no);
        println!("  location: {:?}", upvalue.location);
        println!("  closed: {:?}", upvalue.closed);
        print!("  next: ");
        match &upvalue.next {
            Some(next) => Self::print_upvalue(no, &next.borrow()),
            None => println!(""),
        }
    }

    fn print_upvalues(&self) {
        for (i, upvalue) in self.frame_last().closure.upvalues.iter().enumerate() {
            Self::print_upvalue(i, &upvalue.borrow());
        }
    }

    fn print_open_upvalue(&self) {
        match &self.open_upvalue {
            Some(upvalue) => Self::print_upvalue(100, &upvalue.borrow()),
            None => println!("None"),
        }
    }

    fn close_upvalues(&mut self) {
        let mut current_upvalue: Option<Rc<RefCell<UpvalueObject>>> = self.open_upvalue.clone();
        loop {
            match current_upvalue {
                Some(ref current_upvalue_obj) => {
                    let tmp_location = current_upvalue_obj.borrow().location;
                    let next = current_upvalue_obj.borrow().next.clone();
                    let current_upvalue_location = match tmp_location {
                        Some(loc) => loc,
                        None => {
                            current_upvalue = next;
                            continue;
                        }
                    };
                    let current_upvalue_closed = self.stack.get(current_upvalue_location).clone();
                    current_upvalue_obj.borrow_mut().closed = Some(current_upvalue_closed);
                    current_upvalue_obj.borrow_mut().location = None;
                    current_upvalue = next;
                }
                None => {
                    break;
                }
            }
        }
        self.open_upvalue = current_upvalue;
    }

    fn capture_upvalue(&mut self, index: usize) -> Rc<RefCell<UpvalueObject>> {
        let mut prev_upvalue: Option<Rc<RefCell<UpvalueObject>>> = None;
        let mut current_upvalue: Option<Rc<RefCell<UpvalueObject>>> = self.open_upvalue.clone();
        loop {
            match current_upvalue {
                Some(ref current_upvalue_obj) => {
                    let tmp_location = current_upvalue_obj.borrow().location;
                    let current_upvalue_location = match tmp_location {
                        Some(loc) => loc,
                        None => {
                            let next = current_upvalue_obj.borrow().next.clone();
                            current_upvalue = next;
                            continue;
                        }
                    };
                    if current_upvalue_location > index {
                        prev_upvalue = Some(current_upvalue_obj.clone());
                        let next = current_upvalue_obj.borrow().next.clone();
                        current_upvalue = next;
                    } else {
                        break;
                    }
                }
                None => {
                    break;
                }
            }
        }
        if let Some(ref current_upvalue) = current_upvalue {
            match current_upvalue.borrow().location {
                Some(current_upvalue_location) => {
                    if current_upvalue_location == index {
                        return Rc::clone(current_upvalue);
                    }
                }
                None => {}
            };
        }
        let mut created_upvalue = UpvalueObject::new(index);
        created_upvalue.next = current_upvalue;
        let result = Rc::new(RefCell::new(created_upvalue));
        if let Some(prev_upvalue) = prev_upvalue {
            prev_upvalue.borrow_mut().next = Some(result.clone());
        } else {
            self.open_upvalue = Some(result.clone());
        }
        return result;
    }

    fn frame_push(&mut self, frame: CallFrame) {
        self.frames[self.frame_count] = frame;
        self.frame_count += 1;
    }

    fn frame_pop(&mut self) -> &CallFrame {
        self.frame_count -= 1;
        unsafe { self.frames.get_unchecked(self.frame_count) }
    }

    fn frame_last(&self) -> &CallFrame {
        unsafe { self.frames.get_unchecked(self.frame_count - 1) }
    }

    fn frame_last_mut(&mut self) -> &mut CallFrame {
        unsafe { self.frames.get_unchecked_mut(self.frame_count - 1) }
    }

    fn get_current_ip(&self) -> usize {
        self.frame_last().ip
    }

    fn get_current_ip_mut(&mut self) -> &mut usize {
        &mut self.frame_last_mut().ip
    }

    fn read_constant(frame: &mut CallFrame) -> Value {
        let index: usize = match frame.closure.function.chunk.get_instruction(frame.ip) {
            Some(c) => *c as usize,
            None => panic!(),
        };
        frame.ip += 1;

        frame.closure.function.chunk.get_constant(index)
    }

    fn read_byte(frame: &mut CallFrame) -> Option<u8> {
        let index = match frame.closure.function.chunk.get_instruction(frame.ip) {
            Some(c) => Some(*c),
            None => return None,
        };
        frame.ip += 1;
        index
    }

    fn read_local_index(frame: &mut CallFrame) -> Option<usize> {
        let index: usize = match frame.closure.function.chunk.get_instruction(frame.ip) {
            Some(c) => *c as usize,
            None => return None,
        };
        frame.ip += 1;
        Some(index)
    }

    fn read_jump(frame: &mut CallFrame) -> Option<usize> {
        let index: usize = match frame.closure.function.chunk.read_u16(frame.ip) {
            Some(c) => c as usize,
            None => return None,
        };
        frame.ip += 2;
        Some(index)
    }

    fn read_string(frame: &mut CallFrame) -> Rc<String> {
        match Self::read_constant(frame) {
            Value::String(value) => value,
            _ => panic!(),
        }
    }
}

fn range(n: &[Value]) -> Value {
    if n.len() == 1 {
        let stop = match &n[0] {
            Value::Integer(v) => *v,
            _ => panic!(),
        };
        let mut values: Vec<Value> = Vec::new();
        for i in 0..stop {
            values.push(Value::Integer(i));
        }
        Value::Array(Rc::new(RefCell::new(values)))
    } else if n.len() == 2 {
        let start = match &n[0] {
            Value::Integer(v) => *v,
            _ => panic!(),
        };
        let stop = match &n[1] {
            Value::Integer(v) => *v,
            _ => panic!(),
        };
        let mut values: Vec<Value> = Vec::new();
        for i in start..stop {
            values.push(Value::Integer(i));
        }
        Value::Array(Rc::new(RefCell::new(values)))
    } else if n.len() == 3 {
        let start = match &n[0] {
            Value::Integer(v) => *v,
            _ => panic!(),
        };
        let stop = match &n[1] {
            Value::Integer(v) => *v,
            _ => panic!(),
        };
        let step = match &n[2] {
            Value::Integer(v) => *v as usize,
            _ => panic!(),
        };
        let mut values: Vec<Value> = Vec::new();
        for i in (start..stop).step_by(step) {
            values.push(Value::Integer(i));
        }
        Value::Array(Rc::new(RefCell::new(values)))
    } else {
        Value::Null
    }
}

fn get(n: &[Value]) -> Value {
    if n.len() != 2 {
        return Value::Null;
    }
    let array = match &n[0] {
        Value::Array(v) => v,
        _ => panic!(),
    };
    if let Value::Integer(n) = n[1] {
        match array.borrow().get(n as usize) {
            Some(v) => return v.clone(),
            None => return Value::Null,
        }
    }
    Value::Null
}

fn last(n: &[Value]) -> Value {
    if n.len() != 1 {
        return Value::Null;
    }
    let array = match &n[0] {
        Value::Array(v) => v,
        _ => panic!(),
    };
    return array.borrow().last().unwrap().clone();
}

fn first(n: &[Value]) -> Value {
    if n.len() != 1 {
        return Value::Null;
    }
    let array = match &n[0] {
        Value::Array(v) => v,
        _ => panic!(),
    };
    return array.borrow().first().unwrap().clone();
}

fn rest(n: &[Value]) -> Value {
    if n.len() != 1 {
        return Value::Null;
    }
    let array = match &n[0] {
        Value::Array(v) => v,
        _ => panic!(),
    };
    return Value::Array(Rc::new(RefCell::new(
        array
            .borrow()
            .iter()
            .skip(1)
            .map(|v| v.clone())
            .collect::<Vec<_>>(),
    )));
}

fn str(n: &[Value]) -> Value {
    if n.len() != 1 {
        return Value::Null;
    }
    return Value::String(Rc::new(format!("{}", &n[0])));
}

fn append(n: &[Value]) -> Value {
    if n.len() < 2 {
        return Value::Null;
    }
    let array = match &n[0] {
        Value::Array(v) => (*Rc::clone(v)).clone(),
        _ => panic!(),
    };
    for v in &n[1..] {
        array.borrow_mut().push(v.clone());
    }
    return Value::Array(Rc::new(array));
}

fn len(n: &[Value]) -> Value {
    if n.is_empty() {
        return Value::Null;
    }
    if let Value::Array(n) = &n[0] {
        return Value::Integer(n.borrow().len() as i64);
    }
    return Value::Null;
}

fn now(n: &[Value]) -> Value {
    Value::DateTime(LocalTime::now())
}

