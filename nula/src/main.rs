use std::env;

mod ast;
mod cli;
mod commands;
mod config;
mod generator;
mod cranelift_generator;
mod interpreter;
mod optimizer;
mod parser;
mod translator;
mod utils;

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
        "fmt" => commands::format_project(&args),
        "test" => commands::test_project(&args),
        "create" => commands::create_project(&args),
        "init" => commands::init_project(),
        "install" => commands::install_dependency(&args),
        "remove" => commands::remove_dependency(&args),
        "update" => commands::update_dependencies(),
        "update-nula" => commands::update_nula(),
        "resolve" => commands::resolve_dependencies(),
        _ => cli::print_error(&format!("Unknown command: {}", command)),
    }
}
