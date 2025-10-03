use crate::ast::AstNode;
use crate::cli::print_info;
use std::collections::HashMap;

pub fn interpret_ast(ast: &AstNode) -> f64 {
    print_info("Starting interpretation...");
    let mut variables: HashMap<String, Box<AstNode>> = HashMap::new();
    let result = interpret_node(ast, &mut variables);
    print_info("Interpretation completed");
    result
}

fn interpret_node(node: &AstNode, vars: &mut HashMap<String, Box<AstNode>>) -> f64 {
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
            vars.insert(name.clone(), expr.clone());
            0.0
        }
        AstNode::Assignment(name, expr) => {
            vars.insert(name.clone(), expr.clone());
            0.0
        }
        AstNode::FunctionDef(name, params, body) => {
            vars.insert(name.clone(), Box::new(AstNode::FunctionDef(
                name.clone(),
                                                                    params.clone(),
                                                                    body.clone(),
            )));
            0.0
        }
        AstNode::ForLoop(var, iter, body) => {
            let iter_val = interpret_node(iter, vars) as i32;
            for i in 0..iter_val {
                vars.insert(var.clone(), Box::new(AstNode::NumberLit(i as f64)));
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
            let expr_ref = expr.as_ref();
            match expr_ref {
                AstNode::StringLit(s) => {
                    println!("{}", s);
                    0.0
                }
                AstNode::NumberLit(n) => {
                    println!("{}", n);
                    *n
                }
                AstNode::BoolLit(b) => {
                    println!("{}", b);
                    if *b { 1.0 } else { 0.0 }
                }
                AstNode::Ident(name) => {
                    let val = vars.get(name).cloned();
                    if let Some(val) = val {
                        let val_ref = val.as_ref();
                        match val_ref {
                            AstNode::StringLit(s) => {
                                println!("{}", s);
                                0.0
                            }
                            AstNode::NumberLit(n) => {
                                println!("{}", n);
                                *n
                            }
                            AstNode::BoolLit(b) => {
                                println!("{}", b);
                                if *b { 1.0 } else { 0.0 }
                            }
                            _ => {
                                let result = interpret_node(val_ref, vars);
                                println!("{}", result);
                                result
                            }
                        }
                    } else {
                        println!("undefined");
                        0.0
                    }
                }
                AstNode::Binary(left, op, right) => {
                    let result = interpret_node(&AstNode::Binary(left.clone(), op.clone(), right.clone()), vars);
                    println!("{}", result);
                    result
                }
                AstNode::Call(name, args) => {
                    let result = interpret_node(&AstNode::Call(name.clone(), args.clone()), vars);
                    println!("{}", result);
                    result
                }
                _ => {
                    let result = interpret_node(expr_ref, vars);
                    println!("{}", result);
                    result
                }
            }
        }
        AstNode::Add(left, right) => interpret_node(left, vars) + interpret_node(right, vars),
        AstNode::Mul(left, right) => interpret_node(left, vars) * interpret_node(right, vars),
        AstNode::Return(expr_opt) => {
            if let Some(expr) = expr_opt {
                interpret_node(expr.as_ref(), vars)
            } else {
                0.0
            }
        }
        AstNode::StringLit(_) => 0.0,
        AstNode::NumberLit(num) => *num,
        AstNode::BoolLit(b) => if *b { 1.0 } else { 0.0 },
        AstNode::Ident(name) => {
            let val = vars.get(name).cloned();
            if let Some(val) = val {
                interpret_node(val.as_ref(), vars)
            } else {
                0.0
            }
        }
        AstNode::Call(name, args) => {
            let func_box = vars.get(name).cloned();
            if let Some(func_box) = func_box {
                let func_ref = func_box.as_ref();
                if let AstNode::FunctionDef(_, params, body) = func_ref {
                    let mut local_vars = vars.clone();
                    for (param, arg) in params.iter().zip(args.iter()) {
                        let arg_val = interpret_node(arg, vars);
                        local_vars.insert(param.clone(), Box::new(AstNode::NumberLit(arg_val)));
                    }
                    let mut result = 0.0;
                    for stmt in body {
                        result = interpret_node(stmt, &mut local_vars);
                    }
                    result
                } else {
                    0.0
                }
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
                "/" => if r != 0.0 { l / r } else { 0.0 },
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
