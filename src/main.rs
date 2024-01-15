use crate::compiler::object::FunctionType;
use crate::compiler::parser::Parser as AstParser;
use crate::compiler::Compiler;
use crate::vm::frame::CallFrame;
use clap::Parser;
use compiler::object::ClosureObject;
use compiler::ClassCompiler;
use std::fs::File;
use std::io::Read;
use std::process::ExitCode;
use std::{cell::RefCell, rc::Rc};
use vm::VM;

mod compiler;
mod vm;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    input: Option<String>,
}

fn main() -> ExitCode {
    let args = Args::parse();
    if let Some(input_path) = args.input {
        let mut file = File::open(input_path).expect("file not found");
        let mut data: String = String::new();
        file.read_to_string(&mut data)
            .expect("something went wrong reading the file");

        let mut parser = AstParser::new(&data);
        let program = parser.parse().unwrap();

        let compiler = Rc::new(RefCell::new(Compiler::new(
            "__main__",
            FunctionType::Script,
            0,
            None,
        )));
        let class_compiler = Rc::new(RefCell::new(ClassCompiler::new()));
        for stmt in program.stmts {
            Compiler::compile_stmt(compiler.clone(), class_compiler.clone(), stmt);
        }
        let frame = CallFrame::new(
            Rc::new(ClosureObject::new(Rc::new(
                compiler.borrow().function.clone(),
            ))),
            0,
            0,
        );
        let mut vm = VM::new(frame);
        match vm.interpret() {
            vm::InterpretResult::Ok => {
                return ExitCode::from(0);
            }
            vm::InterpretResult::CompileError => {
                return ExitCode::from(8);
            }
            vm::InterpretResult::RuntimeError(msg) => {
                println!("{}", msg);
                return ExitCode::from(101);
            }
            vm::InterpretResult::End => {
                return ExitCode::from(0);
            }
        }
    } else {
        println!("repl");
        ExitCode::from(0)
    }
}
