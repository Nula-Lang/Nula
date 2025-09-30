use colored::*;
use std::env;

pub fn print_help() {
    println!("\n{}", "Nula CLI - A Modern Programming Language".bold().cyan().underline());
    println!("{}", "=================================".cyan());
    println!("{} {}", "Usage:".bold().white(), "nula <command> [options]".white());
    println!("\n{}:", "Commands".bold().green());
    println!("  {:<12} - Display this help message", "help / ?".yellow());
    println!("  {:<12} - Build project to binary (must be in project dir)", "build".yellow());
    println!("    {:<10} - Build in release mode with optimizations", "--release".magenta());
    println!("    {:<10} - Specify target architecture (e.g., x86_64)", "--target <arch>".magenta());
    println!("  {:<12} - Run a .nula file without compilation", "run [file]".yellow());
    println!("  {:<12} - Create a new Nula project", "create <name>".yellow());
    println!("  {:<12} - Install a dependency from Nula repository", "install <dep>".yellow());
    println!("  {:<12} - Remove an installed dependency", "remove <dep>".yellow());
    println!("  {:<12} - Resolve and install all project dependencies", "resolve".yellow());
    println!("\n{}", "Visit https://nula-lang.github.io/Nula-Website/ for more information".italic().blue());
}

pub fn print_error(msg: &str) {
    eprintln!("{} {}", "error:".bold().red(), msg.white().on_red());
}

pub fn print_success(msg: &str) {
    println!("{} {}", "success:".bold().green(), msg.green());
}

pub fn print_warning(msg: &str) {
    println!("{} {}", "warning:".bold().yellow(), msg.yellow());
}

pub fn print_info(msg: &str) {
    println!("{} {}", "info:".bold().blue(), msg.blue());
}

pub fn print_debug(msg: &str) {
    if env::var("NULA_DEBUG").is_ok() {
        println!("{} {}", "debug:".bold().magenta(), msg.magenta());
    }
}

pub fn print_note(msg: &str) {
    println!("{} {}", "note:".bold().cyan(), msg.cyan());
}

pub fn print_compiling(file: &str) {
    println!("{} {}", "Compiling".bold().purple(), file.purple());
}

pub fn print_parsing(file: &str) {
    println!("{} {}", "Parsing".bold().cyan(), file.cyan());
}

pub fn print_optimizing() {
    println!("{}", "Optimizing AST...".bold().yellow());
}

pub fn print_generating_asm(file: &str) {
    println!("{} {}", "Generating assembly for".bold().green(), file.green());
}

pub fn print_assembling(file: &str) {
    println!("{} {}", "Assembling".bold().blue(), file.blue());
}

pub fn print_linking(file: &str) {
    println!("{} {}", "Linking".bold().magenta(), file.magenta());
}

pub fn print_finished(mode: &str, time: f64) {
    println!("{} {} in {:.2}s", "Finished".bold().green(), mode.green(), time);
}
