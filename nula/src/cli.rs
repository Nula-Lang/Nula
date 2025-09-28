use crate::parser::{NulaParser, Rule};
use crate::ast::AstNode;
use crate::codegen::generate_c_code;
use crate::interpreter::interpret_ast;
use crate::deps::{install_dep, resolve_deps};
use crate::utils::{get_project_dir, show_error};
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::*;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tokio::runtime::Runtime;

#[derive(Parser)]
#[command(name = "nula", about = "Nula Programming Language CLI", version = "0.2.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Help,
    Create { name: String },
    Build,
    Run { file: Option<String> },
    Install { dep: String },
    Test,
    Clean,
    Version,
}

pub fn run_cli() -> Result<()> {
    let cli = Cli::parse();
    let rt = Runtime::new().context("Failed to create runtime")?;

    match cli.command {
        Commands::Help => show_help(),
        Commands::Create { name } => rt.block_on(create_project(&name))?,
        Commands::Build => build_project()?,
        Commands::Run { file } => run_code(file)?,
        Commands::Install { dep } => rt.block_on(install_dep(&dep))?,
        Commands::Test => test_project()?,
        Commands::Clean => clean_project()?,
        Commands::Version => println!("Nula v0.2.0"),
    }
    Ok(())
}

fn show_help() -> Result<()> {
    println!("{}", "Nula CLI Commands".bold().cyan());
    println!("  {} - Show this help", "help".green());
    println!("  {} <name> - Create new project", "create".green());
    println!("  {} - Build to binary", "build".green());
    println!("  {} [file] - Run code", "run".green());
    println!("  {} <dep> - Install dep", "install".green());
    println!("  {} - Run tests", "test".green());
    println!("  {} - Clean artifacts", "clean".green());
    println!("  {} - Show version", "version".green());
    Ok(())
}

async fn create_project(name: &str) -> Result<()> {
    // Jak wcześniej, ale dodaj więcej plików
    let project_dir = PathBuf::from(name);
    fs::create_dir_all(project_dir.join("src"))?;
    fs::create_dir_all(project_dir.join("tests"))?;
    let main_file = project_dir.join("src/main.nula");
    fs::write(&main_file, "let x = 5;\nwrite x;\n#=python={print(\"hi\")}\n")?;
    println!("{} Created project '{}'", "✓".green(), name);
    Ok(())
}

fn build_project() -> Result<()> {
    let project_dir = get_project_dir()?;
    let main_file = project_dir.join("src/main.nula");
    let code = fs::read_to_string(&main_file)?;
    
    let pairs = NulaParser::parse(Rule::program, &code)
        .map_err(|e| show_error(&e.to_string(), &main_file))?;
    
    let ast = crate::parser::parse_to_ast(pairs);
    
    // Resolve deps
    let deps: Vec<_> = ast.iter().filter_map(|n| if let AstNode::Dependency(d) = n { Some(d.clone()) } else { None }).collect();
    resolve_deps(&deps)?;
    
    let c_file = generate_c_code(&ast, &project_dir)?;
    
    // Compile to .s
    let asm_file = project_dir.join("output.s");
    Command::new("gcc").arg("-S").arg(&c_file).arg("-o").arg(&asm_file).output()
        .context("GCC to asm failed")?;
    println!("{} Generated {}", "✓".green(), asm_file.display().to_string().yellow());
    
    // To binary
    let bin_file = project_dir.join("nula_bin");
    let output = Command::new("gcc").arg(&c_file).arg("-o").arg(&bin_file).output()
        .context("GCC compilation failed")?;
    
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr).to_string();
        show_error(&err, &c_file)?;
    }
    
    println!("{} Built {}", "✓".green(), bin_file.display().to_string().green());
    Ok(())
}

fn run_code(file: Option<String>) -> Result<()> {
    // Jak wcześniej, ale z deps i lepszym interpreterem
    let file_path = file.map(PathBuf::from).unwrap_or(get_project_dir()?.join("src/main.nula"));
    let code = fs::read_to_string(&file_path)?;
    let pairs = NulaParser::parse(Rule::program, &code)
        .map_err(|e| show_error(&e.to_string(), &file_path))?;
    let ast = crate::parser::parse_to_ast(pairs);
    interpret_ast(&ast);
    Ok(())
}

fn test_project() -> Result<()> {
    println!("Running tests... (stub)");
    Ok(())
}

fn clean_project() -> Result<()> {
    let dir = get_project_dir()?;
    fs::remove_file(dir.join("output.c")).ok();
    fs::remove_file(dir.join("output.s")).ok();
    fs::remove_file(dir.join("nula_bin")).ok();
    println!("{} Cleaned", "✓".green());
    Ok(())
}
