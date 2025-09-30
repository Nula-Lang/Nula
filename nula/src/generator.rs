use crate::cli::print_debug;

pub fn generate_assembly(ast: &str, release: bool, target: Option<&str>) -> String {
    let mut asm = String::new();
    asm.push_str(".section .text\n.global main\nmain:\n");

    for line in ast.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("//") {
            continue;
        }
        if line.starts_with("write") {
            let msg = line.trim_start_matches("write ").trim_matches(|c| c == '"' || c == '\'');
            asm.push_str("    mov $1, %rax\n"); // Syscall write
            asm.push_str("    mov $1, %rdi\n"); // Stdout
            asm.push_str(&format!("    lea .Lstr{}, %rsi\n", msg.replace(' ', "_")));
            asm.push_str(&format!("    mov ${}, %rdx\n", msg.len()));
            asm.push_str("    syscall\n");
            asm.push_str(&format!(".Lstr{}:\n    .string \"{}\"\n", msg.replace(' ', "_"), msg));
        } else if line.starts_with("add") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() == 3 {
                asm.push_str(&format!("    mov ${}, %rax\n", parts[1]));
                asm.push_str(&format!("    add ${}, %rax\n", parts[2]));
            }
        } else if line.starts_with("mul") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() == 3 {
                asm.push_str(&format!("    mov ${}, %rax\n", parts[1]));
                asm.push_str(&format!("    imul ${}, %rax\n", parts[2]));
            }
        } else if line.starts_with("var") {
            let parts: Vec<&str> = line.split('=').collect();
            if parts.len() == 2 {
                let name = parts[0].trim_start_matches("var ").trim();
                let value = parts[1].trim();
                asm.push_str(&format!(".Lvar_{}: .quad {}\n", name, value));
            }
        }
    }

    asm.push_str("    mov $60, %rax\n    xor %rdi, %rdi\n    syscall\n");

    if release {
        asm.push_str("// Optimized for release\n");
    }
    if let Some(t) = target {
        asm.push_str(&format!("// Targeted for {}\n", t));
    }

    print_debug(&format!("Generated ASM:\n{}", asm));
    asm
}
