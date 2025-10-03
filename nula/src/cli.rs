use colored::*;
use std::env;
use std::path::Path;
use crate::parser::Rule;

pub fn print_help() {
    println!("\n{}", "Nula Programming Language".bold().cyan().underline());
    println!("{}", "=================================".cyan().bold());
    println!("{} {}", "Usage:".bold().bright_white(), "nula [options]".bright_white());
    println!("\n{}:", "Commands".bold().bright_green());
    println!(" {:<12} - Display this help message", "help / ?".bright_yellow());
    println!(" {:<12} - Build project to binary (must be in project dir)", "build".bright_yellow());
    println!(" {:<10} - Build for Windows", "--windows".bright_magenta());
    println!(" {:<10} - Build for Linux", "--linux".bright_magenta());
    println!(" {:<10} - Build in release mode using Cranelift", "--release".bright_magenta());
    println!(" {:<10} - Enable verbose output", "--verbose".bright_magenta());
    println!(" {:<12} - Run a .nula file without compilation", "run [file]".bright_yellow());
    println!(" {:<12} - Check a .nula file for syntax errors", "check [file]".bright_yellow());
    println!(" {:<12} - Format a .nula file", "fmt [file]".bright_yellow());
    println!(" {:<12} - Run tests in the project", "test".bright_yellow());
    println!(" {:<12} - Create a new Nula project", "create ".bright_yellow());
    println!(" {:<12} - Initialize Nula project in current directory", "init".bright_yellow());
    println!(" {:<12} - Install a dependency from Nula repository", "install ".bright_yellow());
    println!(" {:<12} - Remove an installed dependency", "remove ".bright_yellow());
    println!(" {:<12} - Update installed dependencies", "update".bright_yellow());
    println!(" {:<12} - Update Nula binaries to latest version", "update-nula".bright_yellow());
    println!(" {:<12} - Resolve and install all project dependencies", "resolve".bright_yellow());
    println!("\n{}", "Visit https://nula-lang.github.io/Nula-Website/ for more information".italic().bright_blue());
}

pub fn print_error(msg: &str) {
    eprintln!("{} {}", "error:".bold().bright_red(), msg.bright_red());
}

pub fn print_parse_error(err: &pest::error::Error<Rule>, path: &str, code: &str) {
    let line_col = match &err.line_col {
        pest::error::LineColLocation::Pos((line, col)) => (*line, *col),
        pest::error::LineColLocation::Span((start_line, start_col), _) => (*start_line, *start_col),
    };
    let (line, col) = line_col;
    let variant = match &err.variant {
        pest::error::ErrorVariant::ParsingError { positives, negatives } => {
            format!("expected one of {:?}, found {:?}", positives, negatives)
        }
        pest::error::ErrorVariant::CustomError { message } => message.clone(),
    };
    eprintln!("{} {}", "error:".bold().bright_red(), variant.bold().bright_red());
    eprintln!(" {} {}:{}:{}", "-->".bright_blue(), path.bright_blue(), line, col);
    eprintln!(" {} ", "|".bright_blue());
    let lines: Vec<&str> = code.lines().collect();
    if line > 0 && line <= lines.len() {
        let err_line = lines[line - 1];
        eprintln!(" {} {} {}", format!("{:4}", line).bright_blue(), "|".bright_blue(), err_line.bright_white());
        eprintln!(" {} {} {}{}", "", "|".bright_blue(), " ".repeat(col - 1), "^".bold().bright_red());
    }
    eprintln!(" {} ", "|".bright_blue());
}

pub fn print_success(msg: &str) {
    println!("{} {}", "success:".bold().bright_green(), msg.bright_green());
}

pub fn print_warning(msg: &str) {
    println!("{} {}", "warning:".bold().bright_yellow(), msg.bright_yellow());
}

pub fn print_info(msg: &str) {
    println!("{} {}", "info:".bold().bright_blue(), msg.bright_blue());
}

pub fn print_debug(msg: &str) {
    if env::var("NULA_DEBUG").is_ok() {
        println!("{} {}", "debug:".bold().bright_magenta(), msg.bright_magenta());
    }
}

pub fn print_note(msg: &str) {
    println!("{} {}", "note:".bold().bright_cyan(), msg.bright_cyan());
}

pub fn print_compiling(file: &str) {
    println!("{} {}", "Compiling".bold().bright_purple(), Path::new(file).display().to_string().bright_purple());
}

pub fn print_parsing(file: &str) {
    println!("{} {}", "Parsing".bold().bright_cyan(), Path::new(file).display().to_string().bright_cyan());
}

pub fn print_optimizing() {
    println!("{}", "Optimizing AST...".bold().bright_yellow());
}

pub fn print_finished(mode: &str, time: f64) {
    println!("{} {} in {:.2}s", "Finished".bold().bright_green(), mode.bright_green(), time);
}

pub fn print_verbose(msg: &str) {
    if env::var("NULA_VERBOSE").is_ok() {
        println!("{} {}", "verbose:".bold().bright_white(), msg.bright_white());
    }
}
