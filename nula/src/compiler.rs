use crate::parser::NulaAst;
use std::fmt::Write;

pub fn to_assembly(ast: &[NulaAst]) -> anyhow::Result<String> {
    let mut s = String::new();
    s.push_str(".global main\n");

    for node in ast {
        match node {
            NulaAst::Print(s) => {
                writeln!(s, "main:")?;
                writeln!(s, "    mov $1, %rax  // sys_write")?;
                writeln!(s, "    mov ${}, %rdi", s.len() as i64)?;
                writeln!(s, "    mov ${}, %rsi", s.as_ptr() as i64)?;
                writeln!(s, "    mov $1, %rdx")?;
                writeln!(s, "    syscall")?;
            }
            NulaAst::Assign(id, val) => {
                writeln!(s, "    mov ${}, %{}  // assign", val.parse::<i64>().unwrap_or(0), id)?;
            }
            _ => {}
        }
    }
    writeln!(s, "    ret")?;
    Ok(s)
}

pub fn compile_to_binary(s_code: &str) -> anyhow::Result<()> {
    std::fs::write("output.s", s_code)?;
    // Wywołaj nula-zig dla optymalizacji, potem gcc
    std::process::Command::new("nula-zig")
        .arg("output.s")
        .status()?;
    std::process::Command::new("gcc")
        .arg("optimized.s")
        .arg("-o")
        .arg("main")
        .status()?;
    Ok(())
}
