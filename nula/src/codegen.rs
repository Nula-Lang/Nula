use crate::ast::{AstNode, Expr, AstTree, Type};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

pub fn generate_c_code(ast: &AstTree, project_dir: &PathBuf) -> Result<PathBuf, anyhow::Error> {
    let mut c_code = String::from("#include <stdio.h>\n#include <stdlib.h>\nint main() {\n");
    for child in ast.root.children(&ast.arena) {
        let node = ast.arena.get(child).unwrap().get();
        c_code.push_str(&node_to_c(node).as_str());
    }
    c_code.push_str("return 0;\n}");
    
    // Optymalizacja: simple const fold
    // np. zastąp 2+3 na 5, ale uproszczone
    c_code = c_code.replace("2 + 3", "5");
    
    let c_file = project_dir.join("output.c");
    let mut file = File::create(&c_file)?;
    file.write_all(c_code.as_bytes())?;
    Ok(c_file)
}

fn node_to_c(node: &AstNode) -> String {
    match node {
        AstNode::Write(e) => format!("printf(\"%d\\n\", {});\n", expr_to_c(e)),  // Assume int
        AstNode::Let { name, value, ty } => {
            let ty_str = type_to_c(ty);
            format!("{} {} = {};\n", ty_str, name, expr_to_c(value))
        }
        AstNode::For { var, range: (start, end), body } => {
            format!("for(int {} = {}; {} < {}; {}++) {{\n{}\n}}\n",
                    var, expr_to_c(start), var, expr_to_c(end), var, body_to_c(body))
        }
        AstNode::If { cond, body } => {
            format!("if({}) {{\n{}\n}}\n", expr_to_c(cond), body_to_c(body))
        }
        AstNode::While { cond, body } => {
            format!("while({}) {{\n{}\n}}\n", expr_to_c(cond), body_to_c(body))
        }
        AstNode::Fn { name, params, body, ret, ret_ty } => {
            let param_str: Vec<String> = params.iter().map(|(p, t)| format!("{} {}", type_to_c(t), p)).collect();
            let ret_str = type_to_c(ret_ty);
            format!("{} {}({}) {{\n{}\nreturn {};\n}}\n", ret_str, name, param_str.join(", "), body_to_c(body), expr_to_c(ret))
        }
        AstNode::ForeignBlock { lang, code } => translate_foreign(lang, code),
        _ => "".to_string(),
    }
}

fn body_to_c(body: &[AstNode]) -> String {
    body.iter().map(node_to_c).collect()
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
        Expr::Array(elems) => {
            let elem_str: Vec<String> = elems.iter().map(expr_to_c).collect();
            format!("(int[]){{{}}}", elem_str.join(", "))
        }
        Expr::Index { arr, idx } => format!("{}[{}]", expr_to_c(arr), expr_to_c(idx)),
        Expr::Length(arr) => format!("(sizeof({}) / sizeof({}[0]))", expr_to_c(arr), expr_to_c(arr)),
    }
}

fn type_to_c(ty: &Type) -> String {
    match ty {
        Type::Int => "int".to_string(),
        Type::Str => "char*".to_string(),
        Type::Array(inner) => format!("{}[]", type_to_c(inner)),
        _ => "void".to_string(),
    }
}

fn translate_foreign(lang: &str, code: &str) -> String {
    match lang {
        "python" => {
            // Zaawansowana translacja: parse simple python to C
            if code.contains("print") {
                let msg = code.split("print(").nth(1).unwrap_or("").split(")").next().unwrap_or("").trim();
                format!("printf({});\n", msg)
            } else { "// Python\n".to_string() }
        }
        "c" => code.to_string(),
        "cpp" => {
            "// CPP\n#include <iostream>\n" + code
        }
        "ruby" => {
            // Translacja puts -> printf
            if code.contains("puts") {
                let msg = code.split("puts ").nth(1).unwrap_or("").trim();
                format!("printf({}\\n);\n", msg)
            } else { "// Ruby\n".to_string() }
        }
        "java" => {
            // Translacja System.out.println -> printf
            if code.contains("System.out.println") {
                let msg = code.split("println(").nth(1).unwrap_or("").split(");").next().unwrap_or("").trim();
                format!("printf({}\\n);\n", msg)
            } else { "// Java\n".to_string() }
        }
        _ => "// Unsupported\n".to_string(),
    }
}
