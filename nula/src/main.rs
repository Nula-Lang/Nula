mod ast;
mod parser;
mod codegen;
mod interpreter;
mod cli;
mod deps;
mod utils;
mod formatter;

use anyhow::Result;
use cli::run_cli;

fn main() -> Result<()> {
    run_cli()
}
