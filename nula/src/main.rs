use crate::cli::{print_error, print_help};
use crate::commands::{build_project, create_project, install_dependency, remove_dependency, resolve_dependencies, run_project};
use crate::config::load_config;

mod cli;
mod commands;
mod config;
mod generator;
mod interpreter;
mod optimizer;
mod parser;
mod process_expression;
mod translator;
mod utils;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        print_help();
        return;
    }

    let command = &args[1];
    match command.as_str() {
        "help" | "?" => print_help(),
        "build" => {
            let config = match load_config() {
                Ok(v) => v,
                Err(e) => {
                    print_error(&format!("Config error: {}", e));
                    return;
                }
            };
            build_project(&args, &config);
        }
        "run" => run_project(&args),
        "create" => create_project(&args),
        "install" => install_dependency(&args),
        "remove" => remove_dependency(&args),
        "resolve" => resolve_dependencies(),
        _ => print_error(&format!("Unknown command: {}", command)),
    }
}
