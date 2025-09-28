use crate::ast::{AstNode, Expr};
use std::collections::HashMap;

pub fn interpret_ast(ast: &[AstNode]) {
    let mut vars: HashMap<String, Expr> = HashMap::new();
    interpret_nodes(ast, &mut vars);
}

fn interpret_nodes(nodes: &[AstNode], vars: &mut HashMap<String, Expr>) {
    for node in nodes {
        match node {
            AstNode::Write(s) => println!("{}", s),
            AstNode::Let { name, value } => { vars.insert(name.clone(), value.clone()); }
            AstNode::For { var, range: (start, end), body } => {
                for i in *start..*end {
                    vars.insert(var.clone(), Expr::Int(i));
                    interpret_nodes(body, vars);
                }
            }
            AstNode::If { cond, body } => {
                if eval_expr(cond, vars) > Expr::Int(0) {  // Simplify
                    interpret_nodes(body, vars);
                }
            }
            AstNode::Fn { .. } => {}  // Skip def, assume called elsewhere
            AstNode::ForeignBlock { lang, code } => {
                println!("Executing foreign {}: {}", lang, code);
                // Simulate
            }
            _ => {}
        }
    }
}

fn eval_expr(expr: &Expr, vars: &HashMap<String, Expr>) -> Expr {
    match expr {
        Expr::Int(i) => Expr::Int(*i),
        Expr::Var(v) => vars.get(v).cloned().unwrap_or(Expr::Int(0)),
        Expr::BinOp { op, left, right } => {
            let l = eval_expr(left, vars);
            let r = eval_expr(right, vars);
            if let (Expr::Int(li), Expr::Int(ri)) = (&l, &r) {
                match op {
                    '+' => Expr::Int(li + ri),
                    '-' => Expr::Int(li - ri),
                    '*' => Expr::Int(li * ri),
                    '>' => Expr::Int(if li > ri {1} else {0}),
                    _ => Expr::Int(0),
                }
            } else { Expr::Int(0) }
        }
        _ => Expr::Int(0),
    }
}
