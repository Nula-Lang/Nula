use anyhow::{Context, Result};
use clap::Parser;
use colored::*;
use pest::Parser;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Parser)]
#[command(name = "nula", about = "Nula Compiler")]
struct Cli {
    #[arg(short, long)]
    command: Option<String>,
}

#[derive(Parser)]
struct NulaParser {
    #[parser]
    #[grammar = "nula.pest"]  // Generowane z build.rs
}

mod cli;
mod parser;
mod compiler;
mod translator;

use parser::NulaAst;
use compiler::compile_to_binary;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command.as_deref() {
        Some("help") | Some("?") => cli::show_help(),
        Some("build") => build()?,
        Some("run") => run()?,
        Some("create") => create_project()?,
        Some("install") => {
            let dep = env::args().nth(2).context("No dep")?;
            install_dep(&dep)?;
        }
        None => {
            eprintln!("{} No command. Use {}", "Error:".red(), "nula help".green());
        }
        _ => eprintln!("{} Unknown command", "Error:".red()),
    }

    Ok(())
}

fn build() -> Result<()> {
    let cwd = env::current_dir()?;
    if !Path::new("main.nula").exists() {
        return Err(anyhow::anyhow!("Must be in project dir"));
    }

    let code = fs::read_to_string("main.nula").context("Read main.nula")?;
    let pairs = NulaParser::parse(NulaParser::Rule::program, &code)
        .map_err(|e| format!("Parse error: {:?}", e))?
        .next()
        .unwrap();

    let ast = parser::parse_ast(pairs);
    let s_code = compiler::to_assembly(&ast)?;
    fs::write("output.s", &s_code)?;

    // Kompiluj do binarki via gcc (cross-platform: gcc/clang)
    let status = Command::new("gcc")
        .arg("output.s")
        .arg("-o")
        .arg("main")
        .status()?;
    if !status.success() {
        return Err(anyhow::anyhow!("GCC failed"));
    }

    println!("{} Built to main", "Success:".green());
    Ok(())
}

fn run() -> Result<()> {
    // Użyj nula-python dla interpretera
    let status = Command::new("nula-python")
        .arg("main.nula")
        .status()?;
    if !status.success() {
        eprintln!("{} Run failed", "Error:".red());
    }
    Ok(())
}

fn create_project() -> Result<()> {
    let project_name = env::args().nth(2).unwrap_or_else(|| "newproject".to_string());
    fs::create_dir(&project_name)?;
    fs::write(format!("{}/main.nula", project_name), "write(\"Hello Nula!\");")?;
    println!("{} Created {}", "Success:".green(), project_name);
    Ok(())
}

fn install_dep(dep: &str) -> Result<()> {
    // Pobierz library.nula via curl
    let lib_content = Command::new("curl")
        .arg("-s")
        .arg("https://raw.githubusercontent.com/Nula-Lang/Nula/main/library.nula")
        .output()?
        .stdout;

    let lib_str = String::from_utf8(lib_content)?;
    let lines: Vec<&str> = lib_str.lines().collect();

    let mut repo_url = None;
    for line in lines {
        if line.starts_with(&format!("{} -> ", dep)) {
            repo_url = Some(line.split(" -> ").nth(1).unwrap().to_string());
            break;
        }
    }

    let repo = repo_url.context("Dep not found")?;
    let lib_dir = Path::new("/usr/lib/.nula-lib");
    fs::create_dir_all(lib_dir)?;

    // Klonuj via git (cross-platform)
    Command::new("git")
        .current_dir(lib_dir)
        .arg("clone")
        .arg(&repo)
        .status()?;

    println!("{} Installed {}", "Success:".green(), dep);
    Ok(())
}
