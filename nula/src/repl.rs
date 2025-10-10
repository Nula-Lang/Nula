use std::io::{self, Write};
use crate::interpreter::interpret_ast;
use crate::lexer::lex;
use crate::parser::Parser;
pub fn start_repl() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    println!("Nula REPL - type 'exit' or 'quit' to exit.");
    loop {
        print!("nula> ");
        let _ = stdout.flush();
        let mut line = String::new();
        if stdin.read_line(&mut line).unwrap() == 0 {
            break;
        }
        let line = line.trim();
        if line == "exit" || line == "quit" {
            break;
        }
        if line.is_empty() {
            continue;
        }
        let tokens = lex(line);
        let mut parser = Parser::new(tokens);
        match parser.parse_program() {
            Ok(ast) => {
                let result = interpret_ast(&ast);
                println!("{}", result);
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
    println!("Exiting REPL.");
}
