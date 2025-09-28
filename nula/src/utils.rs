use anyhow::Error;
use backtrace::Backtrace;
use colored::*;
use std::path::PathBuf;
use std::env;

pub fn get_project_dir() -> Result<PathBuf, Error> {
    let mut cwd = env::current_dir()?;
    loop {
        if cwd.join("src/main.nula").exists() {
            return Ok(cwd);
        }
        if !cwd.pop() {
            return Err(anyhow::anyhow!("Not in Nula project"));
        }
    }
}

pub fn show_error(msg: &str, file: &PathBuf) -> Error {
    eprintln!("{} in {}:\n{}", "Error".red().bold(), file.display().to_string().yellow(), msg.red());
    eprintln!("Backtrace:\n{:?}", Backtrace::new());
    anyhow::anyhow!(msg.to_string())
}
