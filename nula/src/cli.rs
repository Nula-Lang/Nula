use crate::parser::{NulaParser, Rule};
use crate::ast::AstTree;
use crate::codegen::generate_c_code;
use crate::interpreter::interpret_ast;
use crate::deps::{install_dep, resolve_deps, load_bottles_deps};
use crate::utils::{get_project_dir, show_error};
use crate::formatter::format_code;
use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::*;
use prettytable::{Table, Row, Cell};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tokio::runtime::Runtime;

#[derive(Parser)]
#[command(name = "nula", about = "Nula Programming Language CLI", version = "0.3.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Help,
    Create { name: String },
    Init,
    Build,
    Run { file: Option<String> },
    Install { dep: String },
    Update,
    Test,
    Clean,
    Fmt { file: Option<String> },
    Doc,
    Version,
}

pub fn run_cli() -> Result<()> {
    let cli = Cli::parse();
    let rt = Runtime::new().context("Failed to create runtime")?;

    match cli.command {
        Commands::Help => show_help(),
        Commands::Create { name } => rt.block_on(create_project(&name))?,
        Commands::Init => init_project()?,
        Commands::Build => build_project()?,
        Commands::Run { file } => run_code(file)?,
        Commands::Install { dep } => rt.block_on(install_dep(&dep))?,
        Commands::Update => rt.block_on(update_deps())?,
        Commands::Test => test_project()?,
        Commands::Clean => clean_project()?,
        Commands::Fmt { file } => fmt_code(file)?,
        Commands::Doc => generate_doc()?,
        Commands::Version => println!("Nula v0.3.0"),
    }
    Ok(())
}

fn show_help() -> Result<()> {
    let mut table = Table::new();
    table.add_row(Row::new(vec![
        Cell::new("Command").style_spec("bFc"),
        Cell::new("Description").style_spec("bFc"),
    ]));
    table.add_row(Row::new(vec![Cell::new("help"), Cell::new("Show this help")]));
    table.add_row(Row::new(vec![Cell::new("create <name>"), Cell::new("Create new project")]));
    table.add_row(Row::new(vec![Cell::new("init"), Cell::new("Init project in current dir")]));
    table.add_row(Row::new(vec![Cell::new("build"), Cell::new("Build to binary")]));
    table.add_row(Row::new(vec![Cell::new("run [file]"), Cell::new("Run code")]));
    table.add_row(Row::new(vec![Cell::new("install <dep>"), Cell::new("Install dep")]));
    table.add_row(Row::new(vec![Cell::new("update"), Cell::new("Update all deps")]));
    table.add_row(Row::new(vec![Cell::new("test"), Cell::new("Run tests")]));
    table.add_row(Row::new(vec![Cell::new("clean"), Cell::new("Clean artifacts")]));
    table.add_row(Row::new(vec![Cell::new("fmt [file]"), Cell::new("Format code")]));
    table.add_row(Row::new(vec![Cell::new("doc"), Cell::new("Generate docs")]));
    table.add_row(Row::new(vec![Cell::new("version"), Cell::new("Show version")]));
    table.printstd();
    Ok(())
}

async fn create_project(name: &str) -> Result<()> {
    let project_dir = PathBuf::from(name);
    fs::create_dir_all(project_dir.join("src"))?;
    fs::create_dir_all(project_dir.join("tests"))?;
    let main_file = project_dir.join("src/main.nula");
    fs::write(&main_file, "write Hello Nula;\n")?;
    let bottles = project_dir.join("nula.bottles");
    fs::write(&bottles, ":<ast>:\n:<biblioteka>:\n")?;  // Opcjonalny plik
    println!("{} Created '{}'", "✓".green(), name);
    Ok(())
}

fn init_project() -> Result<()> {
    let dir = env::current_dir()?;
    fs::create_dir_all(dir.join("src"))?;
    fs::create_dir_all(dir.join("tests"))?;
    let main_file = dir.join("src/main.nula");
    if !main_file.exists() {
        fs::write(&main_file, "write Hello Nula;\n")?;
    }
    println!("{} Initialized project", "✓".green());
    Ok(())
}

fn build_project() -> Result<()> {
    let project_dir = get_project_dir()?;
    let main_file = project_dir.join("src/main.nula");
    let code = fs::read_to_string(&main_file)?;
    
    println!("{} Parsing...", "⏳".yellow());
    let pairs = NulaParser::parse(Rule::program, &code)
        .map_err(|e| show_error(&e.to_string(), &main_file, &code))?;
    
    let ast = crate::parser::parse_to_ast(pairs);
    
    println!("{} Resolving deps...", "⏳".yellow());
    let mut deps: Vec<String> = ast.arena.iter().filter_map(|n| {
        if let AstNode::Dependency(d) = n.get() { Some(d.clone()) } else { None }
    }).collect();
    let bottles_deps = load_bottles_deps(&project_dir)?;
    deps.extend(bottles_deps);
    resolve_deps(&deps)?;
    
    println!("{} Generating code...", "⏳".yellow());
    let c_file = generate_c_code(&ast, &project_dir)?;
    
    println!("{} Compiling to asm...", "⏳".yellow());
    let asm_file = project_dir.join("output.s");
    Command::new("gcc").arg("-S").arg(&c_file).arg("-o").arg(&asm_file).output()
        .context("GCC asm failed")?;
    
    println!("{} Compiling to binary...", "⏳".yellow());
    let bin_file = project_dir.join("nula_bin");
    let output = Command::new("gcc").arg(&c_file).arg("-o").arg(&bin_file).output()
        .context("GCC failed")?;
    
    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr).to_string();
        show_error(&err, &c_file, &fs::read_to_string(&c_file)?)?;
    }
    
    println!("{} Built {}", "✓".green(), bin_file.display());
    Ok(())
}

fn run_code(file: Option<String>) -> Result<()> {
    let file_path = file.map(PathBuf::from).unwrap_or(get_project_dir()?.join("src/main.nula"));
    let code = fs::read_to_string(&file_path)?;
    let pairs = NulaParser::parse(Rule::program, &code)
        .map_err(|e| show_error(&e.to_string(), &file_path, &code))?;
    let ast = crate::parser::parse_to_ast(pairs);
    interpret_ast(&ast);
    Ok(())
}

async fn update_deps() -> Result<()> {
    // Stub: reinstall all deps from bottles or inline
    println!("Updating deps... (stub)");
    Ok(())
}

fn test_project() -> Result<()> {
    // Uruchom przykłady/tests
    let dir = get_project_dir()?;
    let test_file = dir.join("tests/test.nula");
    if test_file.exists() {
        run_code(Some(test_file.to_str().unwrap().to_string()))?;
    }
    println!("Tests passed (stub)");
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

fn fmt_code(file: Option<String>) -> Result<()> {
    let file_path = file.map(PathBuf::from).unwrap_or(get_project_dir()?.join("src/main.nula"));
    let code = fs::read_to_string(&file_path)?;
    let formatted = format_code(&code)?;
    fs::write(&file_path, formatted)?;
    println!("{} Formatted {}", "✓".green(), file_path.display());
    Ok(())
}

fn generate_doc() -> Result<()> {
    let dir = get_project_dir()?;
    let main_file = dir.join("src/main.nula");
    let code = fs::read_to_string(&main_file)?;
    let comments: Vec<String> = code.lines().filter(|l| l.starts_with("@")).map(|l| l.to_string()).collect();
    let doc_file = dir.join("docs.md");
    fs::write(&doc_file, comments.join("\n"))?;
    println!("{} Generated docs.md", "✓".green());
    Ok(())
}
