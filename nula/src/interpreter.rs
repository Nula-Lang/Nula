use crate::ast::AstNode;
use crate::cli::print_info;
use std::collections::HashMap;

pub fn interpret_ast(ast: &AstNode) -> f64 {
    print_info("Starting interpretation...");
    let mut variables: HashMap<String, AstNode> = HashMap::new();
    let result = interpret_node(ast, &mut variables);
    print_info("Interpretation completed");
    result
}

fn interpret_node(node: &AstNode, vars: &mut HashMap<String, AstNode>) -> f64 {
    match node {
        AstNode::Program(nodes) => {
            let mut result = 0.0;
            for n in nodes {
                result = interpret_node(n, vars);
            }
            result
        }
        AstNode::Translation(lang, code) => {
            println!("#{}={}", lang, code);
            0.0
        }
        AstNode::Dependency(dep) => {
            println!("<{}>", dep);
            0.0
        }
        AstNode::Import(import) => {
            println!("import {}", import);
            0.0
        }
        AstNode::Comment(_) => 0.0,
        AstNode::VariableDecl(name, expr) => {
            let value = interpret_node(expr, vars);
            vars.insert(name.clone(), AstNode::NumberLit(value));
            0.0
        }
        AstNode::Assignment(name, expr) => {
            let value = interpret_node(expr, vars);
            vars.insert(name.clone(), AstNode::NumberLit(value));
            0.0
        }
        AstNode::FunctionDef(name, params, body) => {
            vars.insert(name.clone(), AstNode::FunctionDef(name.clone(), params.clone(), body.clone()));
            0.0
        }
        AstNode::ForLoop(var, iter, body) => {
            let iter_val = interpret_node(iter, vars) as i32;
            for i in 0..iter_val {
                vars.insert(var.clone(), AstNode::NumberLit(i as f64));
                for stmt in body {
                    interpret_node(stmt, vars);
                }
            }
            0.0
        }
        AstNode::WhileLoop(cond, body) => {
            while interpret_node(cond, vars) != 0.0 {
                for stmt in body {
                    interpret_node(stmt, vars);
                }
            }
            0.0
        }
        AstNode::If(cond, body, else_ifs, else_body) => {
            if interpret_node(cond, vars) != 0.0 {
                for stmt in body {
                    interpret_node(stmt, vars);
                }
            } else {
                let mut executed = false;
                for (ei_cond, ei_body) in else_ifs {
                    if interpret_node(ei_cond, vars) != 0.0 {
                        for stmt in ei_body {
                            interpret_node(stmt, vars);
                        }
                        executed = true;
                        break;
                    }
                }
                if !executed {
                    if let Some(eb) = else_body {
                        for stmt in eb {
                            interpret_node(stmt, vars);
                        }
                    }
                }
            }
            0.0
        }
        AstNode::Write(expr) => {
            let result = interpret_node(expr, vars);
            match expr.as_ref() {
                AstNode::StringLit(s) => {
                    print!("{}", s);
                    0.0
                }
                AstNode::NumberLit(_) => {
                    print!("{}", result);
                    result
                }
                AstNode::BoolLit(b) => {
                    print!("{}", b);
                    if *b { 1.0 } else { 0.0 }
                }
                AstNode::Ident(name) => {
                    let val = vars.get(name).cloned().unwrap_or(AstNode::NumberLit(0.0));
                    let result = interpret_node(&val, vars);
                    print!("{}", result);
                    result
                }
                _ => {
                    print!("{}", result);
                    result
                }
            }
        }
        AstNode::Add(left, right) => interpret_node(left, vars) + interpret_node(right, vars),
        AstNode::Mul(left, right) => interpret_node(left, vars) * interpret_node(right, vars),
        AstNode::Return(expr) => {
            if let Some(e) = expr {
                interpret_node(e, vars)
            } else {
                0.0
            }
        }
        AstNode::StringLit(s) => {
            print!("{}", s);
            0.0
        }
        AstNode::NumberLit(num) => *num,
        AstNode::BoolLit(b) => if *b { 1.0 } else { 0.0 },
        AstNode::Ident(name) => {
            match vars.get(name).cloned().unwrap_or(AstNode::NumberLit(0.0)) {
                AstNode::NumberLit(n) => n,
                AstNode::BoolLit(b) => if b { 1.0 } else { 0.0 },
                other => interpret_node(&other, vars),
            }
        }
        AstNode::Call(name, args) => {
            if let Some(AstNode::FunctionDef(_, params, body)) = vars.get(name).cloned() {
                let mut local_vars = vars.clone();
                for (param, arg) in params.iter().zip(args.iter()) {
                    let arg_val = interpret_node(arg, vars);
                    local_vars.insert(param.clone(), AstNode::NumberLit(arg_val));
                }
                let mut result = 0.0;
                for stmt in body {
                    result = interpret_node(&stmt, &mut local_vars);
                }
                result
            } else {
                0.0
            }
        }
        AstNode::Binary(left, op, right) => {
            let l = interpret_node(left, vars);
            let r = interpret_node(right, vars);
            match op.as_str() {
                "+" => l + r,
                "-" => l - r,
                "*" => l * r,
                "/" => l / r,
                "==" | "eq" => if l == r { 1.0 } else { 0.0 },
                "!=" | "ne" => if l != r { 1.0 } else { 0.0 },
                "<" | "lt" => if l < r { 1.0 } else { 0.0 },
                ">" | "gt" => if l > r { 1.0 } else { 0.0 },
                "<=" | "le" => if l <= r { 1.0 } else { 0.0 },
                ">=" | "ge" => if l >= r { 1.0 } else { 0.0 },
                "and" | "&&" => if l != 0.0 && r != 0.0 { 1.0 } else { 0.0 },
                "or" | "||" => if l != 0.0 || r != 0.0 { 1.0 } else { 0.0 },
                _ => 0.0,
            }
        }
        AstNode::Unary(op, expr) => {
            let val = interpret_node(expr, vars);
            match op.as_str() {
                "-" => -val,
                "not" => if val == 0.0 { 1.0 } else { 0.0 },
                _ => val,
            }
        }
    }
}
