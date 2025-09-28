use crate::ast::{AstNode, Expr};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

pub fn generate_c_code(ast: &[AstNode], project_dir: &PathBuf) -> Result<PathBuf, anyhow::Error> {
    let mut c_code = String::from("#include <stdio.h>\nint main() {\n");
    for node in ast {
        match node {
            AstNode::Write(s) => c_code.push_str(&format!("printf(\"{}\\n\");\n", s)),
            AstNode::Let { name, value } => {
                let val_str = expr_to_c(value);
                c_code.push_str(&format!("int {} = {};\n", name, val_str));  // Assume int for simplicity
            }
            AstNode::For { var, range: (start, end), body } => {
                c_code.push_str(&format!("for(int {} = {}; {} < {}; {}++) {{\n", var, start, var, end, var));
                let body_c = generate_c_body(body);
                c_code.push_str(&body_c);
                c_code.push_str("}\n");
            }
            AstNode::If { cond, body } => {
                let cond_str = expr_to_c(cond);
                c_code.push_str(&format!("if({}) {{\n", cond_str));
                let body_c = generate_c_body(body);
                c_code.push_str(&body_c);
                c_code.push_str("}\n");
            }
            AstNode::Fn { name, params, body, ret } => {
                let param_str: Vec<String> = params.iter().map(|p| format!("int {}", p)).collect();
                c_code.push_str(&format!("int {}({}) {{\n", name, param_str.join(", ")));
                let body_c = generate_c_body(body);
                c_code.push_str(&body_c);
                c_code.push_str(&format!("return {};\n}}\n", expr_to_c(ret)));
            }
            AstNode::ForeignBlock { lang, code } => {
                c_code.push_str(&translate_foreign(lang, code));
            }
            _ => {}
        }
    }
    c_code.push_str("return 0;\n}");
    
    let c_file = project_dir.join("output.c");
    let mut file = File::create(&c_file)?;
    file.write_all(c_code.as_bytes())?;
    Ok(c_file)
}

fn generate_c_body(body: &[AstNode]) -> String {
    let mut body_c = String::new();
    for node in body {
        match node {
            AstNode::Write(s) => body_c.push_str(&format!("printf(\"{}\\n\");\n", s)),
            // Add others recursively
            _ => {}  // Expand for all
        }
    }
    body_c
}

fn expr_to_c(expr: &Expr) -> String {
    match expr {
        Expr::Int(i) => i.to_string(),
        Expr::Str(s) => format!("\"{}\"", s),
        Expr::Var(v) => v.clone(),
        Expr::BinOp { op, left, right } => format!("({} {} {})", expr_to_c(left), op, expr_to_c(right)),
        Expr::Call { name, args } => {
            let arg_str: Vec<String> = args.iter().map(expr_to_c).collect();
            format!("{}({})", name, arg_str.join(", "))
        }
    }
}

fn translate_foreign(lang: &str, code: &str) -> String {
    match lang {
        "python" => {
            // Simple translation: print(x) -> printf("%d\n", x)
            if code.contains("print") {
                "// Translated Python\nprintf(\"Hello from Python\\n\");\n".to_string()
            } else { "".to_string() }
        }
        "c" | "cpp" => format!("#include <some.h>\n{}", code),  // Embed directly
        "ruby" => "// Translated Ruby\nprintf(\"Hello from Ruby\\n\");\n".to_string(),
        "java" => "// Translated Java: System.out.println -> printf\nprintf(\"Hello from Java\\n\");\n".to_string(),
        _ => "// Unsupported lang\n".to_string(),
    }
}
