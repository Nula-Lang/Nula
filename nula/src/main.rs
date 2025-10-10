use std::env;
mod ast;
mod cli;
mod commands;
mod project_commands;
mod config;
mod generator;
mod cranelift_generator;
mod interpreter;
mod optimizer;
mod parser;
mod translator;
mod utils;
mod lexer;
mod repl;
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        cli::print_help();
        return;
    }
    let command = &args[1];
    match command.as_str() {
        "help" | "?" => cli::print_help(),
        "build" => {
            let config = match config::load_config() {
                Ok(v) => v,
                Err(e) => {
                    cli::print_error(&format!("Config error: {}", e));
                    return;
                }
            };
            commands::build_project(&args, &config);
        }
        "run" => commands::run_project(&args),
        "check" => commands::check_project(&args),
        "fmt" => project_commands::format_project(&args),
        "test" => commands::test_project(&args),
        "create" => project_commands::create_project(&args),
        "init" => project_commands::init_project(),
        "install" => project_commands::install_dependency(&args),
        "remove" => project_commands::remove_dependency(&args),
        "update" => project_commands::update_dependencies(),
        "update-nula" => project_commands::update_nula(),
        "resolve" => project_commands::resolve_dependencies(),
        "repl" => commands::repl(&args),
        _ => cli::print_error(&format!("Unknown command: {}", command)),
    }
}
