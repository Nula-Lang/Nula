use colored::*;
use std::env;

pub fn print_help() {
    println!("{}", "Nula CLI Commands:".bold().green());
    println!("  help / ?          - Show this help message");
    println!("  build [options]   - Build the project to binary (must be in project dir)");
    println!("    --release       - Build in release mode with optimizations");
    println!("    --target <arch> - Specify target architecture (e.g., x86_64)");
    println!("  run [file]        - Run .nula file without compilation (may be slower)");
    println!("  create <name>     - Create a new Nula project folder");
    println!("  install <dep>     - Install a dependency from Nula repo");
    println!("  resolve           - Resolve and install all project dependencies");
}

pub fn print_error(msg: &str) {
    eprintln!("{} {}", "Error:".red().bold(), msg.red());
}

pub fn print_success(msg: &str) {
    println!("{} {}", "Success:".green().bold(), msg.green());
}

pub fn print_warning(msg: &str) {
    println!("{} {}", "Warning:".yellow().bold(), msg.yellow());
}

pub fn print_info(msg: &str) {
    println!("{} {}", "Info:".blue().bold(), msg.blue());
}

pub fn print_debug(msg: &str) {
    if env::var("NULA_DEBUG").is_ok() {
        println!("{} {}", "Debug:".magenta().bold(), msg.magenta());
    }
}
