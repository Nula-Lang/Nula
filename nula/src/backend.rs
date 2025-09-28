use std::fs::write;
use std::path::PathBuf;
use std::process::Command;
use colored::*;
use crate::codegen::generate_assembly;
use crate::ir::IrModule;
use crate::utils::show_error;

pub fn compile_to_bin(module: IrModule, project_dir: &PathBuf) -> Result<(), anyhow::Error> {
    let asm_code = generate_assembly(&module);
    let s_file = project_dir.join("output.s");
    write(&s_file, asm_code)?;

    println!("{} Generated assembly: {}", "✓".green().bold(), s_file.display().yellow());

    let bin_file = project_dir.join("nula_bin");
    let output = Command::new("gcc")
        .arg("-c")
        .arg(&s_file)
        .arg("-o")
        .arg(project_dir.join("output.o"))
        .output()?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr).to_string();
        show_error(&err, &s_file, &std::fs::read_to_string(&s_file)?)?;
    }

    let link_output = Command::new("gcc")
        .arg(project_dir.join("output.o"))
        .arg("-o")
        .arg(&bin_file)
        .output()?;

    if !link_output.status.success() {
        let err = String::from_utf8_lossy(&link_output.stderr).to_string();
        show_error(&err, &s_file, "")?;
    }

    println!("{} Built binary: {}", "✓".green().bold(), bin_file.display().green().underline());
    Ok(())
}
