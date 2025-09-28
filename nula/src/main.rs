mod ast;
mod parser;
mod typechecker;
mod ir;
mod codegen;
mod backend;
mod interpreter;
mod cli;
mod deps;
mod utils;
mod formatter;
mod optimizer;

use anyhow::Result;
use cli::run_cli;

#[tokio::main]
async fn main() -> Result<()> {
    run_cli().await
}
