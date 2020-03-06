use std::env;
use std::fs::File;
use std::io::{self, Read, Write};

use lox::vm::{VM, InterpretError};

fn repl() {
    let stdin = io::stdin();
    let mut vm = VM::new();

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        if !stdin.read_line(&mut input).is_ok() {
            eprintln!("Could not read from stdin");
            return;
        }

        match vm.interpret(&input) {
            Ok(()) => (),
            Err(InterpretError::CompileError) => println!("Compile error!"),
            Err(InterpretError::RuntimeError) => println!("Runtime error!"),
        }
    }
}

fn run_file(filename: &str) {
    let mut file = match File::open(filename) {
        Ok(file) => file,
        Err(_) => { eprintln!("Could not find file {}", filename); return }
    };
    let mut source = String::new();
    match file.read_to_string(&mut source) {
        Ok(_) => (),
        Err(_) => { eprintln!("Failed to read from file"); return },
    }

    match VM::new().interpret(&source) {
        Ok(()) => (),
        Err(InterpretError::CompileError) => println!("Compile error!"),
        Err(InterpretError::RuntimeError) => println!("Runtime error!"),
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        repl();
    } else if args.len() == 2 {
        run_file(&args[1]);
    } else {
        println!("Usage: lox [path]");
    }
}
