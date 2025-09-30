use crate::ast::AstNode;
use crate::cli::print_debug;

static mut LABEL_COUNT: usize = 0;

pub fn generate_assembly(ast: &AstNode, release: bool, target: Option<&str>) -> String {
    let mut asm = String::new();
    let mut data = String::new();
    unsafe { LABEL_COUNT = 0; }

    asm.push_str(".global main\n.section .text\nmain:\n");
    asm.push_str("    push %rbp\n");
    asm.push_str("    mov %rsp, %rbp\n");

    generate_node_assembly(ast, &mut asm, &mut data);

    asm.push_str("    mov $0, %rax\n");
    asm.push_str("    pop %rbp\n");
    asm.push_str("    ret\n");

    asm.push_str(&data);

    if release {
        asm.push_str("\n// Release optimizations applied\n");
    }
    if let Some(t) = target {
        asm.push_str(&format!("\n// Target: {}\n", t));
    }

    print_debug(&format!("Generated ASM:\n{}", asm));
    asm
}

fn generate_node_assembly(node: &AstNode, asm: &mut String, data: &mut String) {
    match node {
        AstNode::Program(nodes) => {
            for n in nodes {
                generate_node_assembly(n, asm, data);
            }
        }
        AstNode::Translation(lang, code) => {
            data.push_str(&format!(".data\ntranslation_{}: .string \"{}\"\n", lang, code));
        }
        AstNode::Dependency(dep) => {
            data.push_str(&format!(".data\ndependency: .string \"{}\"\n", dep));
        }
        AstNode::Import(import) => {
            data.push_str(&format!(".data\nimport: .string \"{}\"\n", import));
        }
        AstNode::Comment(comment) => {
            asm.push_str(&format!("    # {}\n", comment));
        }
        AstNode::VariableDecl(name, expr) | AstNode::Assignment(name, expr) => {
            generate_expression_assembly(expr, asm, data);
            let label = format!("var_{}", name);
            asm.push_str(&format!("    mov %rax, {}(%rip)\n", label));
            data.push_str(&format!(".data\n{}: .quad 0\n", label));
        }
        AstNode::FunctionDef(name, _params, body) => {
            asm.push_str(&format!("{}_fn:\n", name));
            asm.push_str("    push %rbp\n");
            asm.push_str("    mov %rsp, %rbp\n");
            for stmt in body {
                generate_node_assembly(stmt, asm, data);
            }
            asm.push_str("    pop %rbp\n");
            asm.push_str("    ret\n");
        }
        AstNode::ForLoop(_var, iter, body) => {
            generate_expression_assembly(iter, asm, data);
            let loop_label = format!("for_loop_{}", unsafe { LABEL_COUNT });
            let body_label = format!("for_body_{}", unsafe { LABEL_COUNT });
            let end_label = format!("for_end_{}", unsafe { LABEL_COUNT });
            unsafe { LABEL_COUNT += 1; }
            asm.push_str("    mov $0, %rcx\n"); // i = 0
            asm.push_str(&format!("{}:\n", loop_label));
            asm.push_str("    cmp %rax, %rcx\n");
            asm.push_str(&format!("    jge {}\n", end_label));
            asm.push_str(&format!("{}:\n", body_label));
            for stmt in body {
                generate_node_assembly(stmt, asm, data);
            }
            asm.push_str("    inc %rcx\n");
            asm.push_str(&format!("    jmp {}\n", loop_label));
            asm.push_str(&format!("{}:\n", end_label));
        }
        AstNode::WhileLoop(cond, body) => {
            let loop_label = format!("while_{}", unsafe { LABEL_COUNT });
            let exit_label = format!("while_exit_{}", unsafe { LABEL_COUNT });
            unsafe { LABEL_COUNT += 1; }
            asm.push_str(&format!("{}:\n", loop_label));
            generate_expression_assembly(cond, asm, data);
            asm.push_str("    cmp $0, %rax\n");
            asm.push_str(&format!("    je {}\n", exit_label));
            for stmt in body {
                generate_node_assembly(stmt, asm, data);
            }
            asm.push_str(&format!("    jmp {}\n", loop_label));
            asm.push_str(&format!("{}:\n", exit_label));
        }
        AstNode::If(cond, body, else_ifs, else_body) => {
            generate_expression_assembly(cond, asm, data);
            let else_label = format!("else_{}", unsafe { LABEL_COUNT });
            let end_label = format!("end_if_{}", unsafe { LABEL_COUNT });
            unsafe { LABEL_COUNT += 1; }
            asm.push_str("    cmp $0, %rax\n");
            asm.push_str(&format!("    je {}\n", else_label));
            for stmt in body {
                generate_node_assembly(stmt, asm, data);
            }
            asm.push_str(&format!("    jmp {}\n", end_label));
            asm.push_str(&format!("{}:\n", else_label));
            for (ei_cond, ei_body) in else_ifs {
                generate_expression_assembly(ei_cond, asm, data);
                let ei_else_label = format!("else_if_else_{}", unsafe { LABEL_COUNT });
                unsafe { LABEL_COUNT += 1; }
                asm.push_str("    cmp $0, %rax\n");
                asm.push_str(&format!("    je {}\n", ei_else_label));
                for stmt in ei_body {
                    generate_node_assembly(stmt, asm, data);
                }
                asm.push_str(&format!("    jmp {}\n", end_label));
                asm.push_str(&format!("{}:\n", ei_else_label));
            }
            if let Some(eb) = else_body {
                for stmt in eb {
                    generate_node_assembly(stmt, asm, data);
                }
            }
            asm.push_str(&format!("{}:\n", end_label));
        }
        AstNode::Write(expr) => {
            generate_expression_assembly(expr, asm, data);
            match **expr {
                AstNode::StringLit(_) => {
                    asm.push_str("    mov $1, %rax\n");
                    asm.push_str("    mov $1, %rdi\n");
                    asm.push_str("    syscall\n");
                }
                AstNode::NumberLit(_) => {
                    asm.push_str("    lea fmt_float(%rip), %rdi\n");
                    asm.push_str("    mov $1, %al\n");
                    asm.push_str("    call printf@plt\n");
                    data.push_str(".data\nfmt_float: .asciz \"%f\\n\"\n");
                }
                AstNode::BoolLit(b) => {
                    let s = if b { "true" } else { "false" };
                    let label = format!("bool_{}", unsafe { LABEL_COUNT });
                    unsafe { LABEL_COUNT += 1; }
                    asm.push_str(&format!("    lea {}(%rip), %rsi\n", label));
                    asm.push_str(&format!("    mov ${}, %rdx\n", s.len()));
                    asm.push_str("    mov $1, %rax\n");
                    asm.push_str("    mov $1, %rdi\n");
                    asm.push_str("    syscall\n");
                    data.push_str(&format!(".data\n{}: .string \"{}\"\n", label, s));
                }
                _ => {}
            }
        }
        AstNode::Add(left, right) => {
            generate_expression_assembly(left, asm, data);
            asm.push_str("    push %rax\n");
            generate_expression_assembly(right, asm, data);
            asm.push_str("    pop %rbx\n");
            asm.push_str("    add %rbx, %rax\n");
        }
        AstNode::Mul(left, right) => {
            generate_expression_assembly(left, asm, data);
            asm.push_str("    push %rax\n");
            generate_expression_assembly(right, asm, data);
            asm.push_str("    pop %rbx\n");
            asm.push_str("    imul %rbx, %rax\n");
        }
        AstNode::Return(expr) => {
            if let Some(e) = expr {
                generate_expression_assembly(e, asm, data);
            }
            asm.push_str("    pop %rbp\n");
            asm.push_str("    ret\n");
        }
        AstNode::Binary(left, op, right) => {
            generate_expression_assembly(left, asm, data);
            asm.push_str("    push %rax\n");
            generate_expression_assembly(right, asm, data);
            asm.push_str("    pop %rbx\n");
            match op.as_str() {
                "+" => asm.push_str("    add %rbx, %rax\n"),
                "-" => asm.push_str("    sub %rax, %rbx\n    mov %rbx, %rax\n"),
                "*" => asm.push_str("    imul %rbx, %rax\n"),
                "/" => asm.push_str("    xchg %rax, %rbx\n    cqo\n    idiv %rbx\n"),
                "==" | "eq" => {
                    asm.push_str("    cmp %rax, %rbx\n");
                    asm.push_str("    sete %al\n");
                    asm.push_str("    movzb %al, %rax\n");
                }
                "!=" | "ne" => {
                    asm.push_str("    cmp %rax, %rbx\n");
                    asm.push_str("    setne %al\n");
                    asm.push_str("    movzb %al, %rax\n");
                }
                "<" | "lt" => {
                    asm.push_str("    cmp %rax, %rbx\n");
                    asm.push_str("    setl %al\n");
                    asm.push_str("    movzb %al, %rax\n");
                }
                ">" | "gt" => {
                    asm.push_str("    cmp %rax, %rbx\n");
                    asm.push_str("    setg %al\n");
                    asm.push_str("    movzb %al, %rax\n");
                }
                "<=" | "le" => {
                    asm.push_str("    cmp %rax, %rbx\n");
                    asm.push_str("    setle %al\n");
                    asm.push_str("    movzb %al, %rax\n");
                }
                ">=" | "ge" => {
                    asm.push_str("    cmp %rax, %rbx\n");
                    asm.push_str("    setge %al\n");
                    asm.push_str("    movzb %al, %rax\n");
                }
                "and" | "&&" => {
                    asm.push_str("    and %rbx, %rax\n");
                }
                "or" | "||" => {
                    asm.push_str("    or %rbx, %rax\n");
                }
                _ => {}
            }
        }
        AstNode::Unary(op, expr) => {
            generate_expression_assembly(expr, asm, data);
            match op.as_str() {
                "-" => asm.push_str("    neg %rax\n"),
                "not" => {
                    asm.push_str("    cmp $0, %rax\n");
                    asm.push_str("    sete %al\n");
                    asm.push_str("    movzb %al, %rax\n");
                }
                _ => {}
            }
        }
        _ => {}
    }
}

fn generate_expression_assembly(node: &AstNode, asm: &mut String, data: &mut String) {
    match node {
        AstNode::StringLit(s) => {
            let label = format!("str_{}", unsafe { LABEL_COUNT });
            unsafe { LABEL_COUNT += 1; }
            asm.push_str(&format!("    lea {}(%rip), %rsi\n", label));
            asm.push_str(&format!("    mov ${}, %rdx\n", s.len()));
            data.push_str(&format!(".data\n{}: .string \"{}\"\n", label, s));
        }
        AstNode::NumberLit(num) => {
            asm.push_str(&format!("    mov ${}, %rax\n", *num as i64));
        }
        AstNode::BoolLit(b) => {
            asm.push_str(&format!("    mov ${}, %rax\n", if *b { 1 } else { 0 }));
        }
        AstNode::Ident(name) => {
            let label = format!("var_{}", name);
            asm.push_str(&format!("    mov {}(%rip), %rax\n", label));
        }
        AstNode::Call(name, args) => {
            for arg in args.iter().rev() {
                generate_expression_assembly(arg, asm, data);
                asm.push_str("    push %rax\n");
            }
            asm.push_str(&format!("    call {}_fn\n", name));
            asm.push_str(&format!("    add ${}, %rsp\n", args.len() * 8));
        }
        AstNode::Binary(left, op, right) => {
            generate_expression_assembly(left, asm, data);
            asm.push_str("    push %rax\n");
            generate_expression_assembly(right, asm, data);
            asm.push_str("    pop %rbx\n");
            match op.as_str() {
                "+" => asm.push_str("    add %rbx, %rax\n"),
                "-" => asm.push_str("    sub %rax, %rbx\n    mov %rbx, %rax\n"),
                "*" => asm.push_str("    imul %rbx, %rax\n"),
                "/" => asm.push_str("    xchg %rax, %rbx\n    cqo\n    idiv %rbx\n"),
                "==" | "eq" => {
                    asm.push_str("    cmp %rax, %rbx\n");
                    asm.push_str("    sete %al\n");
                    asm.push_str("    movzb %al, %rax\n");
                }
                "!=" | "ne" => {
                    asm.push_str("    cmp %rax, %rbx\n");
                    asm.push_str("    setne %al\n");
                    asm.push_str("    movzb %al, %rax\n");
                }
                "<" | "lt" => {
                    asm.push_str("    cmp %rax, %rbx\n");
                    asm.push_str("    setl %al\n");
                    asm.push_str("    movzb %al, %rax\n");
                }
                ">" | "gt" => {
                    asm.push_str("    cmp %rax, %rbx\n");
                    asm.push_str("    setg %al\n");
                    asm.push_str("    movzb %al, %rax\n");
                }
                "<=" | "le" => {
                    asm.push_str("    cmp %rax, %rbx\n");
                    asm.push_str("    setle %al\n");
                    asm.push_str("    movzb %al, %rax\n");
                }
                ">=" | "ge" => {
                    asm.push_str("    cmp %rax, %rbx\n");
                    asm.push_str("    setge %al\n");
                    asm.push_str("    movzb %al, %rax\n");
                }
                "and" | "&&" => {
                    asm.push_str("    and %rbx, %rax\n");
                }
                "or" | "||" => {
                    asm.push_str("    or %rbx, %rax\n");
                }
                _ => {}
            }
        }
        AstNode::Unary(op, expr) => {
            generate_expression_assembly(expr, asm, data);
            match op.as_str() {
                "-" => asm.push_str("    neg %rax\n"),
                "not" => {
                    asm.push_str("    cmp $0, %rax\n");
                    asm.push_str("    sete %al\n");
                    asm.push_str("    movzb %al, %rax\n");
                }
                _ => {}
            }
        }
        _ => {}
    }
}
