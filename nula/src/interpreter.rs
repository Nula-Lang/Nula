use crate::ast::{AstNode, Expr, Type};
use std::collections::HashMap;

#[derive(Clone)]
enum Value {
    Int(i32),
    Str(String),
    Array(Vec<Value>),
}

pub fn interpret_ast(ast: &AstTree) {
    let mut vars: HashMap<String, Value> = HashMap::new();
    for child in ast.root.children(&ast.arena) {
        let node = ast.arena.get(child).unwrap().get();
        interpret_node(node, &mut vars);
    }
}

fn interpret_node(node: &AstNode, vars: &mut HashMap<String, Value>) {
    match node {
        AstNode::Write(e) => println!("{}", value_to_str(&eval_expr(e, vars))),
        AstNode::Let { name, value, .. } => {
            vars.insert(name.clone(), eval_expr(value, vars));
        }
        AstNode::For { var, range: (start, end), body } => {
            let s = if let Value::Int(si) = eval_expr(start, vars) { si } else { 0 };
            let e = if let Value::Int(ei) = eval_expr(end, vars) { ei } else { 0 };
            for i in s..e {
                vars.insert(var.clone(), Value::Int(i));
                for b in body {
                    interpret_node(b, vars);
                }
            }
        }
        AstNode::If { cond, body } => {
            if let Value::Int(c) = eval_expr(cond, vars) {
                if c != 0 {
                    for b in body {
                        interpret_node(b, vars);
                    }
                }
            }
        }
        AstNode::While { cond, body } => {
            while let Value::Int(c) = eval_expr(cond, vars) {
                if c == 0 { break; }
                for b in body {
                    interpret_node(b, vars);
                }
            }
        }
        AstNode::Fn { .. } => {} // Def, skip
        AstNode::ForeignBlock { lang, code } => {
            println!("Foreign {}: {}", lang, code); // Simulate exec
        }
        _ => {}
    }
}

fn eval_expr(expr: &Expr, vars: &HashMap<String, Value>) -> Value {
    match expr {
        Expr::Int(i) => Value::Int(*i),
        Expr::Str(s) => Value::Str(s.clone()),
        Expr::Var(v) => vars.get(v).cloned().unwrap_or(Value::Int(0)),
        Expr::BinOp { op, left, right } => {
            let l = eval_expr(left, vars);
            let r = eval_expr(right, vars);
            if let (Value::Int(li), Value::Int(ri)) = (&l, &r) {
                match op {
                    '+' => Value::Int(li + ri),
                    '-' => Value::Int(li - ri),
                    '*' => Value::Int(li * ri),
                    '>' => Value::Int(if li > ri {1} else {0}),
                    '<' => Value::Int(if li < ri {1} else {0}),
                    _ => Value::Int(0),
                }
            } else { Value::Int(0) }
        }
        Expr::Call { .. } => Value::Int(0), // Stub
        Expr::Array(elems) => Value::Array(elems.iter().map(|e| eval_expr(e, vars)).collect()),
        Expr::Index { arr, idx } => {
            if let (Value::Array(a), Value::Int(i)) = (eval_expr(arr, vars), eval_expr(idx, vars)) {
                a.get(i as usize).cloned().unwrap_or(Value::Int(0))
            } else { Value::Int(0) }
        }
        Expr::Length(arr) => {
            if let Value::Array(a) = eval_expr(arr, vars) {
                Value::Int(a.len() as i32)
            } else { Value::Int(0) }
        }
    }
}

fn value_to_str(val: &Value) -> String {
    match val {
        Value::Int(i) => i.to_string(),
        Value::Str(s) => s.clone(),
        Value::Array(a) => format!("[{}]", a.iter().map(value_to_str).collect::<Vec<_>>().join(", ")),
    }
}
