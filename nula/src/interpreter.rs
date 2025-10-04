use crate::ast::AstNode;
use crate::cli::print_info;
use std::collections::HashMap;

pub fn interpret_ast(ast: &AstNode) -> f64 {
    println!("DEBUG: Starting interpretation of AST: {:?}", ast);
    print_info("Starting interpretation...");
    let mut variables: HashMap<String, Box<AstNode>> = HashMap::new();
    let result = interpret_node(ast, &mut variables);
    println!("DEBUG: Interpretation result: {}", result);
    print_info("Interpretation completed");
    result
}

fn interpret_node(node: &AstNode, vars: &mut HashMap<String, Box<AstNode>>) -> f64 {
    println!("DEBUG: Interpreting node: {:?}", node);
    match node {
        AstNode::Program(nodes) => {
            println!("DEBUG: Program with {} nodes", nodes.len());
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
            println!("DEBUG: Declared variable {} = {:?}", name, expr);
            0.0
        }
        AstNode::Assignment(name, expr) => {
            vars.insert(name.clone(), expr.clone());
            println!("DEBUG: Assigned variable {} = {:?}", name, expr);
            0.0
        }
        AstNode::FunctionDef(name, params, body) => {
            vars.insert(name.clone(), Box::new(AstNode::FunctionDef(
                name.clone(),
                                                                    params.clone(),
                                                                    body.clone(),
            )));
            println!("DEBUG: Defined function {} with {} params", name, params.len());
            0.0
        }
        AstNode::ForLoop(var, iter, body) => {
            let iter_val = interpret_node(iter.as_ref(), vars) as i32;
            println!("DEBUG: For loop {} in 0..{} with {} body statements", var, iter_val, body.len());
            for i in 0..iter_val {
                vars.insert(var.clone(), Box::new(AstNode::NumberLit(i as f64)));
                for stmt in body {
                    interpret_node(&stmt, vars);
                }
            }
            0.0
        }
        AstNode::WhileLoop(cond, body) => {
            println!("DEBUG: While loop with {} body statements", body.len());
            while interpret_node(cond.as_ref(), vars) != 0.0 {
                for stmt in body {
                    interpret_node(&stmt, vars);
                }
            }
            0.0
        }
        AstNode::If(cond, body, else_ifs, else_body) => {
            let cond_val = interpret_node(cond.as_ref(), vars);
            println!("DEBUG: If cond = {}, body len = {}, else_ifs = {}, else_body = {:?}", cond_val, body.len(), else_ifs.len(), else_body.is_some());
            if cond_val != 0.0 {
                for stmt in body {
                    interpret_node(&stmt, vars);
                }
            } else {
                let mut executed = false;
                for (ei_cond, ei_body) in else_ifs {
                    if interpret_node(ei_cond.as_ref(), vars) != 0.0 {
                        for stmt in ei_body {
                            interpret_node(&stmt, vars);
                        }
                        executed = true;
                        break;
                    }
                }
                if !executed {
                    if let Some(eb) = else_body {
                        for stmt in eb {
                            interpret_node(&stmt, vars);
                        }
                    }
                }
            }
            0.0
        }
        AstNode::Write(expr) => {
            println!("DEBUG: Write statement with expr: {:?}", expr);
            let expr_ref = expr.as_ref();
            let result = match expr_ref {
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
                    if let Some(val) = vars.get(name).cloned() {
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
                                let res = interpret_node(val_ref, vars);
                                println!("{}", res);
                                res
                            }
                        }
                    } else {
                        println!("undefined");
                        0.0
                    }
                }
                AstNode::Binary(left, op, right) => {
                    let bin_node = AstNode::Binary(left.clone(), op.clone(), right.clone());
                    let res = interpret_node(&bin_node, vars);
                    println!("{}", res);
                    res
                }
                AstNode::Call(name, args) => {
                    let call_node = AstNode::Call(name.clone(), args.clone());
                    let res = interpret_node(&call_node, vars);
                    println!("{}", res);
                    res
                }
                _ => {
                    let res = interpret_node(expr_ref, vars);
                    println!("{}", res);
                    res
                }
            };
            result
        }
        AstNode::Add(left, right) => {
            let l = interpret_node(left.as_ref(), vars);
            let r = interpret_node(right.as_ref(), vars);
            l + r
        }
        AstNode::Mul(left, right) => {
            let l = interpret_node(left.as_ref(), vars);
            let r = interpret_node(right.as_ref(), vars);
            l * r
        }
        AstNode::Return(expr_opt) => {
            if let Some(expr) = expr_opt {
                interpret_node(expr.as_ref(), vars)
            } else {
                0.0
            }
        }
        AstNode::StringLit(s) => {
            println!("DEBUG: String lit: {}", s);
            0.0
        }
        AstNode::NumberLit(num) => *num,
        AstNode::BoolLit(b) => if *b { 1.0 } else { 0.0 },
        AstNode::Ident(name) => {
            if let Some(val) = vars.get(name).cloned() {
                interpret_node(val.as_ref(), vars)
            } else {
                0.0
            }
        }
        AstNode::Call(name, args) => {
            if let Some(func_box) = vars.get(name).cloned() {
                if let AstNode::FunctionDef(_, params, body) = func_box.as_ref() {
                    let mut local_vars = vars.clone();
                    for (param, arg) in params.iter().zip(args.iter()) {
                        let arg_val = interpret_node(arg, vars);
                        local_vars.insert(param.clone(), Box::new(AstNode::NumberLit(arg_val)));
                    }
                    let mut result = 0.0;
                    for stmt in body.iter() {
                        let temp = interpret_node(stmt, &mut local_vars);
                        result = temp;
                        if let AstNode::Return(_) = *stmt {
                            break;
                        }
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
            let l = interpret_node(left.as_ref(), vars);
            let r = interpret_node(right.as_ref(), vars);
            match op.as_str() {
                "+" => l + r,
                "-" => l - r,
                "*" => l * r,
                "/" => if r != 0.0 { l / r } else { 0.0 },
                "==" | "eq" => if (l - r).abs() < f64::EPSILON { 1.0 } else { 0.0 },
                "!=" | "ne" => if (l - r).abs() >= f64::EPSILON { 1.0 } else { 0.0 },
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
            let val = interpret_node(expr.as_ref(), vars);
            match op.as_str() {
                "-" => -val,
                "not" => if val == 0.0 { 1.0 } else { 0.0 },
                _ => val,
            }
        }
    }
}
