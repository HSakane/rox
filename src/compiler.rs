use self::{
    ast::{ExpressionNode, StatementNode},
    object::{FunctionObject, FunctionType},
    scope::{Local, Upvalue},
};
use crate::vm::{
    chunk::{
        OP_ADD, OP_ARRAY, OP_CALL, OP_CLASS, OP_CLOSE_UPVALUE, OP_CLOSURE, OP_CONSTANT, OP_COUNTUP,
        OP_DEFINE_GLOBAL, OP_DIVIDE, OP_EQUAL, OP_FALSE, OP_GET_GLOBAL, OP_GET_LOCAL, OP_GET_PROP,
        OP_GET_SUPER, OP_GET_UPVALUE, OP_GREATER, OP_INDEX_CALL, OP_INDEX_SET, OP_INHERIT,
        OP_CONSTANT0, OP_INVOKE, OP_JUMP, OP_JUMP_IF_FALSE, OP_JUMP_IF_RANGE_END, OP_LESS,
        OP_LOOP, OP_METHOD, OP_MULTIPLY, OP_NEGATIVE, OP_NOT, OP_NULL, OP_POP, OP_POW, OP_PRINT,
        OP_RANGE, OP_REM, OP_RETURN, OP_SET_GLOBAL, OP_SET_LOCAL, OP_SET_PROP, OP_SET_UPVALUE,
        OP_SUBTRACT, OP_SUPER_INVOKE, OP_TRUE,
    },
    value::Value,
};
use core::panic;
use std::{cell::RefCell, rc::Rc};

pub mod ast;
pub mod object;
pub mod parser;
pub mod scanner;
pub mod scope;
pub mod token;

const LOCAL_MAX: usize = 256;
const UPVALUE_MAX: usize = 32;

#[derive(Debug, PartialEq, PartialOrd)]
pub struct ClassCompiler {
    pub enclosing: Option<Rc<RefCell<ClassCompiler>>>,
    pub has_super_class: bool,
}

impl ClassCompiler {
    pub fn new() -> Self {
        Self {
            enclosing: None,
            has_super_class: false,
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd)]
pub struct Compiler {
    pub enclosing: Option<Rc<RefCell<Compiler>>>,
    pub function: FunctionObject,
    pub function_type: FunctionType,
    pub locals: [Local; LOCAL_MAX],
    pub upvalues: [Upvalue; UPVALUE_MAX],
    pub scope_depth: i32,
    pub local_count: usize,
}

impl Compiler {
    pub fn new(
        name: impl Into<String>,
        function_type: FunctionType,
        arity: i32,
        enclosing: Option<Rc<RefCell<Compiler>>>,
    ) -> Self {
        let locals = vec![Local::new("", 0); LOCAL_MAX];
        let upvalues = vec![Upvalue::new(0, false); UPVALUE_MAX];
        let mut compiler = Self {
            enclosing,
            function: FunctionObject::new(name, arity),
            function_type,
            locals: locals.try_into().unwrap(),
            upvalues: upvalues.try_into().unwrap(),
            scope_depth: 0,
            local_count: 0,
        };
        let local = match &compiler.function_type {
            FunctionType::Function => Local::new("", 0),
            _ => Local::new("this", 0),
        };
        let local_count = compiler.local_count;
        compiler.locals[local_count] = local;
        compiler.local_count += 1;
        compiler
    }

    fn function(
        compiler: Rc<RefCell<Compiler>>,
        class_compiler: Rc<RefCell<ClassCompiler>>,
        ftype: FunctionType,
        name: ExpressionNode,
        params: Vec<ExpressionNode>,
        body: Box<StatementNode>,
    ) {
        let name = match name {
            ExpressionNode::Identifer(name) => name,
            _ => todo!(),
        };
        let new_compiler = Rc::new(RefCell::new(Compiler::new(
            &name,
            ftype.clone(),
            params.len() as i32,
            Some(Rc::clone(&compiler)),
        )));
        Self::begin_scope(Rc::clone(&new_compiler));
        for param in &params {
            let param_name = match param {
                ExpressionNode::Identifer(name) => name,
                _ => todo!(),
            };
            if Self::get_scope_depth(Rc::clone(&new_compiler)) > 0 {
                Self::add_local(Rc::clone(&new_compiler), param_name).unwrap();
            }
        }
        Self::compile_stmt(Rc::clone(&new_compiler), class_compiler.clone(), *body);

        match &ftype {
            FunctionType::Init => Self::emit_bytes(Rc::clone(&new_compiler), OP_GET_LOCAL, 0),
            _ => Self::emit_byte(Rc::clone(&new_compiler), OP_NULL),
        }
        Self::emit_byte(Rc::clone(&new_compiler), OP_RETURN);
        Self::end_scope(Rc::clone(&new_compiler));

        let index = compiler
            .borrow_mut()
            .function
            .chunk
            .add_constant(Value::Function(Rc::new(
                new_compiler.borrow().function.clone(),
            )));
        Self::emit_bytes(Rc::clone(&compiler), OP_CLOSURE, index);

        {
            let range = 0..new_compiler.borrow().function.upvalue_count;
            let upvalues = &new_compiler.borrow().upvalues;
            for index in range {
                Self::emit_bytes(
                    Rc::clone(&compiler),
                    if upvalues[index].is_local { 1u8 } else { 0u8 },
                    upvalues[index].index as u8,
                );
            }
        }

        let index = compiler
            .borrow_mut()
            .function
            .chunk
            .add_constant(Value::String(Rc::new(name)));

        match ftype {
            FunctionType::Function => {
                Self::emit_bytes(Rc::clone(&compiler), OP_DEFINE_GLOBAL, index)
            }
            FunctionType::Script => Self::emit_bytes(Rc::clone(&compiler), OP_DEFINE_GLOBAL, index),
            FunctionType::Method => Self::emit_bytes(Rc::clone(&compiler), OP_METHOD, index),
            FunctionType::Init => Self::emit_bytes(Rc::clone(&compiler), OP_METHOD, index),
        };
    }

    pub fn compile_stmt(
        compiler: Rc<RefCell<Compiler>>,
        class_compiler: Rc<RefCell<ClassCompiler>>,
        stmt: StatementNode,
    ) {
        match stmt {
            StatementNode::Class {
                name: class_name,
                body: class_body,
                super_class,
            } => {
                let name = match class_name {
                    ExpressionNode::Identifer(name) => name,
                    _ => todo!(),
                };
                let index = compiler
                    .borrow_mut()
                    .function
                    .chunk
                    .add_constant(Value::String(Rc::new(name.clone())));

                if Self::get_scope_depth(Rc::clone(&compiler)) > 0 {
                    Self::add_local(Rc::clone(&compiler), &name).unwrap();
                    Self::emit_bytes(Rc::clone(&compiler), OP_CLASS, index);
                } else {
                    Self::emit_bytes(Rc::clone(&compiler), OP_CLASS, index);
                    Self::emit_bytes(Rc::clone(&compiler), OP_DEFINE_GLOBAL, index);
                }

                let mut new_class_compiler = ClassCompiler::new();
                new_class_compiler.enclosing = Some(class_compiler.clone());
                let new_class_compiler = Rc::new(RefCell::new(new_class_compiler));

                match super_class {
                    Some(super_class) => {
                        let super_class_name = match super_class {
                            ExpressionNode::Identifer(name) => name,
                            _ => todo!(),
                        };

                        // 自分自身を継承していないかチェック
                        if &super_class_name == &name {
                            panic!("A class can't inherit from itself.({})", &name);
                        }

                        // namedVariable
                        if let Some(index) =
                            Self::get_local(Rc::clone(&compiler), &super_class_name)
                        {
                            Self::emit_bytes(Rc::clone(&compiler), OP_GET_LOCAL, index);
                        } else if let Some(index) =
                            Self::get_upvalue(Rc::clone(&compiler), &super_class_name)
                        {
                            Self::emit_bytes(Rc::clone(&compiler), OP_GET_UPVALUE, index);
                        } else {
                            let index = compiler
                                .borrow_mut()
                                .function
                                .chunk
                                .add_constant(Value::String(Rc::new(super_class_name)));
                            Self::emit_bytes(Rc::clone(&compiler), OP_GET_GLOBAL, index);
                        }

                        // ------
                        Self::begin_scope(Rc::clone(&compiler));
                        Self::add_local(Rc::clone(&compiler), "super").unwrap();
                        // ------

                        // namedVariable
                        if let Some(index) = Self::get_local(Rc::clone(&compiler), &name) {
                            Self::emit_bytes(Rc::clone(&compiler), OP_GET_LOCAL, index);
                        } else if let Some(index) = Self::get_upvalue(Rc::clone(&compiler), &name) {
                            Self::emit_bytes(Rc::clone(&compiler), OP_GET_UPVALUE, index);
                        } else {
                            let index = compiler
                                .borrow_mut()
                                .function
                                .chunk
                                .add_constant(Value::String(Rc::new(name.clone())));
                            Self::emit_bytes(Rc::clone(&compiler), OP_GET_GLOBAL, index);
                        }

                        Self::emit_byte(Rc::clone(&compiler), OP_INHERIT);
                        new_class_compiler.borrow_mut().has_super_class = true;
                    }
                    None => {}
                }

                if let Some(index) = Self::get_local(Rc::clone(&compiler), &name) {
                    Self::emit_bytes(Rc::clone(&compiler), OP_GET_LOCAL, index);
                } else if let Some(index) = Self::get_upvalue(Rc::clone(&compiler), &name) {
                    Self::emit_bytes(Rc::clone(&compiler), OP_GET_UPVALUE, index);
                } else {
                    let index = compiler
                        .borrow_mut()
                        .function
                        .chunk
                        .add_constant(Value::String(Rc::new(name)));
                    Self::emit_bytes(Rc::clone(&compiler), OP_GET_GLOBAL, index);
                }

                match *class_body {
                    StatementNode::Block { stmts } => {
                        for stmt in stmts {
                            match stmt {
                                StatementNode::Fun {
                                    name: method_name,
                                    params: method_params,
                                    body: method_body,
                                } => {
                                    let ftype: FunctionType = match &method_name {
                                        ExpressionNode::Identifer(n) => {
                                            if n == "init" {
                                                FunctionType::Init
                                            } else {
                                                FunctionType::Method
                                            }
                                        }
                                        _ => todo!(),
                                    };
                                    Self::function(
                                        compiler.clone(),
                                        new_class_compiler.clone(),
                                        ftype,
                                        method_name,
                                        method_params,
                                        method_body,
                                    );
                                }
                                invalid => panic!("invalid node. {}", invalid),
                            }
                        }
                    }
                    invalid => panic!("invalid node. {}", invalid),
                }
                Self::emit_byte(Rc::clone(&compiler), OP_POP);
                if new_class_compiler.borrow().has_super_class {
                    Self::end_scope(Rc::clone(&compiler));
                }
            }
            StatementNode::For {
                name,
                range,
                consequence,
            } => {
                // 独自実装で自信なし。より良いやり方確認要
                Self::begin_scope(Rc::clone(&compiler));
                Self::emit_byte(Rc::clone(&compiler), OP_CONSTANT0);
                Self::add_local(Rc::clone(&compiler), "__range_counter__").unwrap();

                let start_loop = {
                    let chunk = &compiler.borrow().function.chunk;
                    chunk.get_instruction_len()
                };

                if let Some(index) = Self::get_local(Rc::clone(&compiler), "__range_counter__") {
                    Self::emit_bytes(Rc::clone(&compiler), OP_GET_LOCAL, index);
                }
                Self::compile_exp(Rc::clone(&compiler), class_compiler.clone(), range);
                let exit_jump = Self::emit_jump(Rc::clone(&compiler), OP_JUMP_IF_RANGE_END);
                if let Some(index) = Self::get_local(Rc::clone(&compiler), "__range_counter__") {
                    Self::emit_bytes(Rc::clone(&compiler), OP_COUNTUP, index);
                }
                // -- ローカル変数定義 --
                let name = match name {
                    ExpressionNode::Identifer(name) => name,
                    _ => todo!(),
                };
                Self::add_local(Rc::clone(&compiler), name).unwrap();
                // -- ローカル変数定義 --

                Self::compile_stmt(Rc::clone(&compiler), class_compiler.clone(), *consequence);
                Self::emit_byte(Rc::clone(&compiler), OP_POP);
                Self::emit_loop(Rc::clone(&compiler), start_loop).unwrap();
                Self::patch_jump(Rc::clone(&compiler), exit_jump).unwrap();
                Self::end_scope(Rc::clone(&compiler));
            }
            StatementNode::Fun { name, params, body } => {
                Self::function(
                    compiler.clone(),
                    class_compiler.clone(),
                    FunctionType::Function,
                    name,
                    params,
                    body,
                );
            }
            StatementNode::If {
                condition: condtion,
                consequence,
                alternative: alternatives,
            } => {
                Self::compile_exp(Rc::clone(&compiler), class_compiler.clone(), condtion);

                let then_jump = Self::emit_jump(Rc::clone(&compiler), OP_JUMP_IF_FALSE);
                Self::emit_byte(Rc::clone(&compiler), OP_POP);
                Self::compile_stmt(Rc::clone(&compiler), class_compiler.clone(), *consequence);
                let else_jump = Self::emit_jump(Rc::clone(&compiler), OP_JUMP);
                Self::patch_jump(Rc::clone(&compiler), then_jump).unwrap();
                Self::emit_byte(Rc::clone(&compiler), OP_POP);
                match alternatives {
                    Some(alternatives) => Self::compile_stmt(
                        Rc::clone(&compiler),
                        class_compiler.clone(),
                        *alternatives,
                    ),
                    None => {}
                };
                Self::patch_jump(Rc::clone(&compiler), else_jump).unwrap();
            }
            StatementNode::Return { value } => {
                match value {
                    Some(exp) => {
                        Self::compile_exp(Rc::clone(&compiler), class_compiler.clone(), exp)
                    }
                    None => Self::compile_exp(
                        Rc::clone(&compiler),
                        class_compiler.clone(),
                        ExpressionNode::NullLiteral,
                    ),
                }
                Self::emit_byte(Rc::clone(&compiler), OP_RETURN);
            }
            StatementNode::Var { name, value } => {
                Self::compile_exp(Rc::clone(&compiler), class_compiler.clone(), value);

                let name = match name {
                    ExpressionNode::Identifer(name) => name,
                    _ => todo!(),
                };

                if Self::get_scope_depth(Rc::clone(&compiler)) > 0 {
                    Self::add_local(Rc::clone(&compiler), name).unwrap();
                    return;
                }
                let index = compiler
                    .borrow_mut()
                    .function
                    .chunk
                    .add_constant(Value::String(Rc::new(name)));
                Self::emit_bytes(compiler, OP_DEFINE_GLOBAL, index);
            }
            StatementNode::While {
                condition: condtion,
                consequence,
            } => {
                let start_loop = {
                    let chunk = &compiler.borrow().function.chunk;
                    chunk.get_instruction_len()
                };
                Self::compile_exp(Rc::clone(&compiler), class_compiler.clone(), condtion);

                let exit_jump = Self::emit_jump(Rc::clone(&compiler), OP_JUMP_IF_FALSE);
                Self::emit_byte(Rc::clone(&compiler), OP_POP);
                Self::compile_stmt(Rc::clone(&compiler), class_compiler.clone(), *consequence);
                Self::emit_loop(Rc::clone(&compiler), start_loop).unwrap();
                Self::patch_jump(Rc::clone(&compiler), exit_jump).unwrap();
                Self::emit_byte(Rc::clone(&compiler), OP_POP);
            }
            StatementNode::Block { stmts } => {
                Self::begin_scope(Rc::clone(&compiler));
                for stmt in stmts {
                    Self::compile_stmt(Rc::clone(&compiler), class_compiler.clone(), stmt);
                }
                Self::end_scope(Rc::clone(&compiler));
            }
            StatementNode::Print { expression } => {
                Self::compile_exp(Rc::clone(&compiler), class_compiler.clone(), expression);
                Self::emit_byte(Rc::clone(&compiler), OP_PRINT);
            }
            StatementNode::ExpStmt { expression } => {
                Self::compile_exp(Rc::clone(&compiler), class_compiler.clone(), expression);
                Self::emit_byte(Rc::clone(&compiler), OP_POP);
            }
        }
    }

    pub fn compile_exp(
        compiler: Rc<RefCell<Compiler>>,
        class_compiler: Rc<RefCell<ClassCompiler>>,
        expression: ExpressionNode,
    ) {
        match expression {
            ExpressionNode::Identifer(name) => {
                if name == "this" && class_compiler.borrow().enclosing.is_none() {
                    panic!("identifer \"this\". but no class.");
                }

                if let Some(index) = Self::get_local(Rc::clone(&compiler), &name) {
                    Self::emit_bytes(Rc::clone(&compiler), OP_GET_LOCAL, index);
                    return;
                }
                if let Some(index) = Self::get_upvalue(Rc::clone(&compiler), &name) {
                    Self::emit_bytes(Rc::clone(&compiler), OP_GET_UPVALUE, index);
                    return;
                }
                let index = compiler
                    .borrow_mut()
                    .function
                    .chunk
                    .add_constant(Value::String(Rc::new(name)));
                Self::emit_bytes(compiler, OP_GET_GLOBAL, index);
            }
            ExpressionNode::StringLiteral(value) => {
                let index = compiler
                    .borrow_mut()
                    .function
                    .chunk
                    .add_constant(Value::String(Rc::new(value)));
                Self::emit_bytes(compiler, OP_CONSTANT, index);
            }
            ExpressionNode::FloatLiteral(value) => {
                let index = compiler
                    .borrow_mut()
                    .function
                    .chunk
                    .add_constant(Value::Float(value));
                Self::emit_bytes(compiler, OP_CONSTANT, index);
            }
            ExpressionNode::IntegerLiteral(value) => {
                let index = compiler
                    .borrow_mut()
                    .function
                    .chunk
                    .add_constant(Value::Integer(value));
                Self::emit_bytes(compiler, OP_CONSTANT, index);
            }
            ExpressionNode::BooleanLiteral(value) => {
                if value {
                    Self::emit_byte(Rc::clone(&compiler), OP_TRUE);
                } else {
                    Self::emit_byte(Rc::clone(&compiler), OP_FALSE);
                }
            }
            ExpressionNode::ArrayLiteral(value) => {
                let length = value.len();
                for exp in value {
                    Self::compile_exp(Rc::clone(&compiler), class_compiler.clone(), exp);
                }
                Self::emit_bytes(Rc::clone(&compiler), OP_ARRAY, length as u8);
            }
            ExpressionNode::RangeLiteral { start, end } => {
                Self::compile_exp(Rc::clone(&compiler), class_compiler.clone(), *start);
                Self::compile_exp(Rc::clone(&compiler), class_compiler.clone(), *end);
                Self::emit_byte(Rc::clone(&compiler), OP_RANGE);
            }
            ExpressionNode::NullLiteral => Self::emit_byte(compiler, OP_NULL),
            ExpressionNode::Prefix { ope, right } => {
                Self::compile_exp(Rc::clone(&compiler), class_compiler.clone(), *right);
                match ope.as_str() {
                    "-" => Self::emit_byte(Rc::clone(&compiler), OP_NEGATIVE),
                    "!" => Self::emit_byte(Rc::clone(&compiler), OP_NOT),
                    _ => {}
                }
            }
            ExpressionNode::Infix { ope, left, right } => {
                Self::compile_exp(Rc::clone(&compiler), class_compiler.clone(), *left);
                Self::compile_exp(Rc::clone(&compiler), class_compiler.clone(), *right);
                match ope.as_str() {
                    "+" => Self::emit_byte(Rc::clone(&compiler), OP_ADD),
                    "-" => Self::emit_byte(Rc::clone(&compiler), OP_SUBTRACT),
                    "*" => Self::emit_byte(Rc::clone(&compiler), OP_MULTIPLY),
                    "/" => Self::emit_byte(Rc::clone(&compiler), OP_DIVIDE),
                    "^" => Self::emit_byte(Rc::clone(&compiler), OP_POW),
                    "%" => Self::emit_byte(Rc::clone(&compiler), OP_REM),
                    "!=" => {
                        Self::emit_byte(Rc::clone(&compiler), OP_EQUAL);
                        Self::emit_byte(Rc::clone(&compiler), OP_NOT);
                    }
                    "==" => Self::emit_byte(Rc::clone(&compiler), OP_EQUAL),
                    ">" => Self::emit_byte(Rc::clone(&compiler), OP_GREATER),
                    ">=" => {
                        Self::emit_byte(Rc::clone(&compiler), OP_LESS);
                        Self::emit_byte(Rc::clone(&compiler), OP_NOT);
                    }
                    "<" => Self::emit_byte(Rc::clone(&compiler), OP_LESS),
                    "<=" => {
                        Self::emit_byte(Rc::clone(&compiler), OP_GREATER);
                        Self::emit_byte(Rc::clone(&compiler), OP_NOT);
                    }
                    _ => {}
                }
            }
            ExpressionNode::GetProperty { left, right } => {
                Self::compile_exp(Rc::clone(&compiler), class_compiler.clone(), *left);
                match &*right {
                    ExpressionNode::Identifer(name) => {
                        let index = compiler
                            .borrow_mut()
                            .function
                            .chunk
                            .add_constant(Value::String(Rc::new(name.clone())));
                        Self::emit_bytes(Rc::clone(&compiler), OP_GET_PROP, index);
                    }
                    _ => todo!(),
                }
            }
            ExpressionNode::GetSuperProperty { left: _, right } => {
                // namedVariable
                let this_name = "this".to_string();
                if let Some(index) = Self::get_local(Rc::clone(&compiler), &this_name) {
                    Self::emit_bytes(Rc::clone(&compiler), OP_GET_LOCAL, index);
                } else if let Some(index) = Self::get_upvalue(Rc::clone(&compiler), &this_name) {
                    Self::emit_bytes(Rc::clone(&compiler), OP_GET_UPVALUE, index);
                } else {
                    let index = compiler
                        .borrow_mut()
                        .function
                        .chunk
                        .add_constant(Value::String(Rc::new(this_name)));
                    Self::emit_bytes(Rc::clone(&compiler), OP_GET_GLOBAL, index);
                }

                match &*right {
                    ExpressionNode::Identifer(name) => {
                        let index = compiler
                            .borrow_mut()
                            .function
                            .chunk
                            .add_constant(Value::String(Rc::new(name.clone())));

                        // namedVariable
                        let super_name: String = "super".to_string();
                        if let Some(index) = Self::get_local(Rc::clone(&compiler), &super_name) {
                            Self::emit_bytes(Rc::clone(&compiler), OP_GET_LOCAL, index);
                        } else if let Some(index) =
                            Self::get_upvalue(Rc::clone(&compiler), &super_name)
                        {
                            Self::emit_bytes(Rc::clone(&compiler), OP_GET_UPVALUE, index);
                        } else {
                            let index = compiler
                                .borrow_mut()
                                .function
                                .chunk
                                .add_constant(Value::String(Rc::new(super_name)));
                            Self::emit_bytes(Rc::clone(&compiler), OP_GET_GLOBAL, index);
                        }

                        Self::emit_bytes(Rc::clone(&compiler), OP_GET_SUPER, index);
                    }
                    _ => todo!(),
                }
            }
            ExpressionNode::SetProperty { left: _, right: _ } => {
                // nop
            }
            ExpressionNode::InvokeMethod {
                left,
                right,
                arguments,
            } => {
                Self::compile_exp(Rc::clone(&compiler), class_compiler.clone(), *left);
                match &*right {
                    ExpressionNode::Identifer(name) => {
                        let len = arguments.len() as u8;
                        for arg in arguments {
                            Self::compile_exp(Rc::clone(&compiler), class_compiler.clone(), arg);
                        }

                        let index = compiler
                            .borrow_mut()
                            .function
                            .chunk
                            .add_constant(Value::String(Rc::new(name.clone())));
                        Self::emit_bytes(Rc::clone(&compiler), OP_INVOKE, index);
                        Self::emit_byte(Rc::clone(&compiler), len);
                    }
                    _ => todo!(),
                }
            }
            ExpressionNode::InvokeSuperMethod {
                left: _,
                right,
                arguments,
            } => {
                // namedVariable
                let this_name = "this".to_string();
                if let Some(index) = Self::get_local(Rc::clone(&compiler), &this_name) {
                    Self::emit_bytes(Rc::clone(&compiler), OP_GET_LOCAL, index);
                } else if let Some(index) = Self::get_upvalue(Rc::clone(&compiler), &this_name) {
                    Self::emit_bytes(Rc::clone(&compiler), OP_GET_UPVALUE, index);
                } else {
                    let index = compiler
                        .borrow_mut()
                        .function
                        .chunk
                        .add_constant(Value::String(Rc::new(this_name)));
                    Self::emit_bytes(Rc::clone(&compiler), OP_GET_GLOBAL, index);
                }

                match &*right {
                    ExpressionNode::Identifer(name) => {
                        let len = arguments.len() as u8;
                        for arg in arguments {
                            Self::compile_exp(Rc::clone(&compiler), class_compiler.clone(), arg);
                        }

                        let index = compiler
                            .borrow_mut()
                            .function
                            .chunk
                            .add_constant(Value::String(Rc::new(name.clone())));

                        // namedVariable
                        let super_name: String = "super".to_string();
                        if let Some(index) = Self::get_local(Rc::clone(&compiler), &super_name) {
                            Self::emit_bytes(Rc::clone(&compiler), OP_GET_LOCAL, index);
                        } else if let Some(index) =
                            Self::get_upvalue(Rc::clone(&compiler), &super_name)
                        {
                            Self::emit_bytes(Rc::clone(&compiler), OP_GET_UPVALUE, index);
                        } else {
                            let index = compiler
                                .borrow_mut()
                                .function
                                .chunk
                                .add_constant(Value::String(Rc::new(super_name)));
                            Self::emit_bytes(Rc::clone(&compiler), OP_GET_GLOBAL, index);
                        }

                        Self::emit_bytes(Rc::clone(&compiler), OP_SUPER_INVOKE, index);
                        Self::emit_byte(Rc::clone(&compiler), len);
                    }
                    _ => todo!(),
                }
            }
            ExpressionNode::Logical { ope, left, right } => match ope.as_str() {
                "and" => {
                    Self::compile_exp(Rc::clone(&compiler), class_compiler.clone(), *left);
                    let end_jump = Self::emit_jump(Rc::clone(&compiler), OP_JUMP_IF_FALSE);
                    Self::emit_byte(Rc::clone(&compiler), OP_POP);
                    Self::compile_exp(Rc::clone(&compiler), class_compiler.clone(), *right);
                    Self::patch_jump(Rc::clone(&compiler), end_jump).unwrap();
                }
                "or" => {
                    Self::compile_exp(Rc::clone(&compiler), class_compiler.clone(), *left);
                    let else_jump = Self::emit_jump(Rc::clone(&compiler), OP_JUMP_IF_FALSE);
                    let end_jump = Self::emit_jump(Rc::clone(&compiler), OP_JUMP);

                    Self::patch_jump(Rc::clone(&compiler), else_jump).unwrap();
                    Self::emit_byte(Rc::clone(&compiler), OP_POP);
                    Self::compile_exp(Rc::clone(&compiler), class_compiler.clone(), *right);
                    Self::patch_jump(Rc::clone(&compiler), end_jump).unwrap();
                }
                _ => {}
            },
            ExpressionNode::Assign { ope, left, right } => match ope.as_str() {
                "=" => match *left {
                    ExpressionNode::Identifer(name) => {
                        Self::compile_exp(Rc::clone(&compiler), class_compiler.clone(), *right);

                        let name = name.clone();
                        if let Some(index) = Self::get_local(Rc::clone(&compiler), &name) {
                            Self::emit_bytes(Rc::clone(&compiler), OP_SET_LOCAL, index);
                            return;
                        }
                        if let Some(index) = Self::get_upvalue(Rc::clone(&compiler), &name) {
                            Self::emit_bytes(Rc::clone(&compiler), OP_SET_UPVALUE, index);
                            return;
                        }
                        let index = compiler
                            .borrow_mut()
                            .function
                            .chunk
                            .add_constant(Value::String(Rc::new(name)));
                        Self::emit_bytes(compiler, OP_SET_GLOBAL, index);
                    }
                    ExpressionNode::SetProperty {
                        left: prop_left,
                        right: prop_right,
                    } => {
                        Self::compile_exp(Rc::clone(&compiler), class_compiler.clone(), *prop_left);
                        match &*prop_right {
                            ExpressionNode::Identifer(name) => {
                                let index = compiler
                                    .borrow_mut()
                                    .function
                                    .chunk
                                    .add_constant(Value::String(Rc::new(name.clone())));
                                Self::compile_exp(
                                    Rc::clone(&compiler),
                                    class_compiler.clone(),
                                    *right,
                                );
                                Self::emit_bytes(Rc::clone(&compiler), OP_SET_PROP, index);
                            }
                            _ => todo!(),
                        }
                    }
                    ExpressionNode::IndexCall { array, index } => {
                        Self::compile_exp(Rc::clone(&compiler), class_compiler.clone(), *array);
                        Self::compile_exp(Rc::clone(&compiler), class_compiler.clone(), *index);
                        Self::compile_exp(Rc::clone(&compiler), class_compiler.clone(), *right);
                        Self::emit_byte(Rc::clone(&compiler), OP_INDEX_SET);
                    }
                    invalid => panic!("invalid node {:?}", invalid),
                },
                _ => {}
            },
            ExpressionNode::FunCall {
                function,
                arguments,
            } => {
                Self::compile_exp(Rc::clone(&compiler), class_compiler.clone(), *function);

                let len = arguments.len() as u8;
                for arg in arguments {
                    Self::compile_exp(Rc::clone(&compiler), class_compiler.clone(), arg);
                }
                Self::emit_bytes(Rc::clone(&compiler), OP_CALL, len);
            }
            ExpressionNode::IndexCall { array, index } => {
                Self::compile_exp(Rc::clone(&compiler), class_compiler.clone(), *array);
                Self::compile_exp(Rc::clone(&compiler), class_compiler.clone(), *index);
                Self::emit_byte(Rc::clone(&compiler), OP_INDEX_CALL);
            }
        }
    }

    fn emit_byte(compiler: Rc<RefCell<Compiler>>, byte: u8) {
        compiler.borrow_mut().function.chunk.write(byte, 0);
    }

    fn emit_bytes(compiler: Rc<RefCell<Compiler>>, byte1: u8, byte2: u8) {
        Self::emit_byte(Rc::clone(&compiler), byte1);
        Self::emit_byte(Rc::clone(&compiler), byte2);
    }

    fn emit_jump(compiler: Rc<RefCell<Compiler>>, byte: u8) -> usize {
        Self::emit_byte(Rc::clone(&compiler), byte);
        Self::emit_bytes(Rc::clone(&compiler), 0xff, 0xff);
        compiler.borrow_mut().function.chunk.get_instruction_len() - 2
    }

    fn patch_jump(compiler: Rc<RefCell<Compiler>>, offset: usize) -> Result<(), String> {
        let jmp = compiler.borrow().function.chunk.get_instruction_len() - offset - 2;
        if jmp > u16::MAX as usize {
            return Err(format!("Too much code to jump over({}).", jmp));
        }

        if let Some(instruction) = compiler
            .borrow_mut()
            .function
            .chunk
            .get_instruction_mut(offset)
        {
            *instruction = ((jmp >> 8) & 0xff) as u8;
        } else {
            return Err(format!("Not found instruction({}).", offset));
        }

        if let Some(instruction) = compiler
            .borrow_mut()
            .function
            .chunk
            .get_instruction_mut(offset + 1)
        {
            *instruction = (jmp & 0xff) as u8;
        } else {
            return Err(format!("Not found instruction({}).", offset + 1));
        }
        Ok(())
    }

    fn emit_loop(compiler: Rc<RefCell<Compiler>>, start_loop: usize) -> Result<(), String> {
        Self::emit_byte(Rc::clone(&compiler), OP_LOOP);
        let offset = compiler.borrow_mut().function.chunk.get_instruction_len() - start_loop + 2;
        if offset > u16::MAX as usize {
            return Err(format!("Too much code to jump over({}).", offset));
        }
        let offset = offset as u16;
        Self::emit_byte(Rc::clone(&compiler), (offset >> 8 & 0xff) as u8);
        Self::emit_byte(Rc::clone(&compiler), (offset & 0xff) as u8);
        Ok(())
    }

    fn begin_scope(compiler: Rc<RefCell<Compiler>>) {
        compiler.borrow_mut().scope_depth += 1;
    }

    fn end_scope(compiler: Rc<RefCell<Compiler>>) {
        compiler.borrow_mut().scope_depth -= 1;
        let range = (0..compiler.borrow().local_count).rev();
        for index in range {
            let local_depth = compiler.borrow().locals[index].depth;
            let is_captured = compiler.borrow().locals[index].is_captured;
            let scope_depth = compiler.borrow().scope_depth;

            if local_depth > scope_depth {
                if is_captured {
                    Self::emit_byte(Rc::clone(&compiler), OP_CLOSE_UPVALUE);
                } else {
                    Self::emit_byte(Rc::clone(&compiler), OP_POP);
                }
                compiler.borrow_mut().local_count -= 1;
            }
        }
    }

    fn get_scope_depth(compiler: Rc<RefCell<Compiler>>) -> i32 {
        compiler.borrow().scope_depth
    }

    fn print_locals(compiler: Rc<RefCell<Compiler>>) {
        for index in (0..compiler.borrow().local_count) {
            let local = &compiler.borrow().locals[index];
            print!("[{:?}] - ", local);
        }
        println!("");
    }

    fn add_local(compiler: Rc<RefCell<Compiler>>, name: impl Into<String>) -> Result<(), String> {
        if compiler.borrow().local_count >= compiler.borrow().locals.len() {
            return Err(format!(
                "add_local over upper.({} >= {})",
                compiler.borrow().local_count,
                compiler.borrow().locals.len()
            ));
        }

        let name = name.into();
        for index in (0..compiler.borrow().local_count).rev() {
            let local = &compiler.borrow().locals[index];
            if local.depth != -1 && local.depth < compiler.borrow().scope_depth {
                break;
            }
            if local.name == name {
                return Err(format!("duplecate identifer.({})", name));
            }
        }

        let local_count = compiler.borrow().local_count;
        let local = Local::new(name, compiler.borrow().scope_depth);
        compiler.borrow_mut().locals[local_count] = local;
        compiler.borrow_mut().local_count += 1;
        Ok(())
    }

    fn get_local(compiler: Rc<RefCell<Compiler>>, name: impl Into<String>) -> Option<u8> {
        let name = name.into();
        for index in (0..compiler.borrow().local_count).rev() {
            let local = &compiler.borrow().locals[index];
            if local.name == name {
                return Some(index as u8);
            }
        }
        None
    }

    fn get_upvalue(compiler: Rc<RefCell<Compiler>>, name: impl Into<String>) -> Option<u8> {
        let name = name.into();
        let enclosing = match compiler.borrow().enclosing {
            Some(ref e) => Rc::clone(e),
            None => return None,
        };

        let local_index = Self::get_local(Rc::clone(&enclosing), &name);
        match local_index {
            Some(local_index) => {
                enclosing.borrow_mut().locals[local_index as usize].is_captured = true;
                return Self::add_upvalue(Rc::clone(&compiler), local_index as usize, true);
            }
            None => {}
        };

        let upvalue_index = Self::get_upvalue(Rc::clone(&enclosing), &name);
        match upvalue_index {
            Some(upvalue_index) => {
                Self::add_upvalue(Rc::clone(&compiler), upvalue_index as usize, false)
            }
            None => None,
        }
    }

    fn add_upvalue(compiler: Rc<RefCell<Compiler>>, index: usize, is_local: bool) -> Option<u8> {
        let upvalue_count = compiler.borrow().function.upvalue_count;
        let upvalue_range = 0..upvalue_count;
        for upvalue_index in upvalue_range {
            if compiler.borrow().upvalues[upvalue_index].index == index
                && compiler.borrow().upvalues[upvalue_index].is_local == is_local
            {
                return Some(upvalue_index as u8);
            }
        }

        compiler.borrow_mut().upvalues[upvalue_count] = Upvalue::new(index, is_local);
        compiler.borrow_mut().function.upvalue_count += 1;
        Some(upvalue_count as u8)
    }

    fn to_owned_function(self) -> FunctionObject {
        self.function
    }
}
