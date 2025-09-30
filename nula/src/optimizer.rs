use crate::ast::AstNode;
use crate::cli::print_debug;

pub fn optimize_ast(ast: AstNode) -> AstNode {
    let optimized = optimize_node(ast);
    print_debug(&format!("Optimized AST: {:?}", optimized));
    optimized
}

fn optimize_node(node: AstNode) -> AstNode {
    match node {
        AstNode::Program(nodes) => {
            let mut opt_nodes = vec![];
            for n in nodes {
                let opt = optimize_node(n);
                if !is_dead_code(&opt) {
                    opt_nodes.push(opt);
                }
            }
            AstNode::Program(opt_nodes)
        }
        AstNode::Translation(lang, code) => AstNode::Translation(lang, code),
        AstNode::Dependency(dep) => AstNode::Dependency(dep),
        AstNode::Import(import) => AstNode::Import(import),
        AstNode::Comment(comment) => AstNode::Comment(comment),
        AstNode::VariableDecl(name, expr) => AstNode::VariableDecl(name, Box::new(optimize_node(*expr))),
        AstNode::FunctionDef(name, params, body) => {
            let opt_body = body.into_iter().map(optimize_node).collect();
            AstNode::FunctionDef(name, params, opt_body)
        }
        AstNode::ForLoop(var, iter, body) => {
            let opt_iter = optimize_node(*iter);
            let opt_body = body.into_iter().map(optimize_node).collect();
            AstNode::ForLoop(var, Box::new(opt_iter), opt_body)
        }
        AstNode::WhileLoop(cond, body) => {
            let opt_cond = optimize_node(*cond);
            let opt_body = body.into_iter().map(optimize_node).collect();
            AstNode::WhileLoop(Box::new(opt_cond), opt_body)
        }
        AstNode::If(cond, body, else_ifs, else_body) => {
            let opt_cond = optimize_node(*cond);
            let opt_body = body.into_iter().map(optimize_node).collect();
            let opt_else_ifs = else_ifs.into_iter().map(|(c, b)| (Box::new(optimize_node(*c)), b.into_iter().map(optimize_node).collect())).collect();
            let opt_else_body = else_body.map(|b| b.into_iter().map(optimize_node).collect());
            AstNode::If(Box::new(opt_cond), opt_body, opt_else_ifs, opt_else_body)
        }
        AstNode::Write(expr) => AstNode::Write(Box::new(optimize_node(*expr))),
        AstNode::Add(left, right) => {
            let left_opt = optimize_node(*left);
            let right_opt = optimize_node(*right);
            if let (AstNode::NumberLit(0.0), _) = (&left_opt, &right_opt) {
                right_opt
            } else if let (_, AstNode::NumberLit(0.0)) = (&left_opt, &right_opt) {
                left_opt
            } else if let (AstNode::NumberLit(a), AstNode::NumberLit(b)) = (&left_opt, &right_opt) {
                AstNode::NumberLit(a + b)
            } else {
                AstNode::Add(Box::new(left_opt), Box::new(right_opt))
            }
        }
        AstNode::Mul(left, right) => {
            let left_opt = optimize_node(*left);
            let right_opt = optimize_node(*right);
            if let (AstNode::NumberLit(1.0), _) = (&left_opt, &right_opt) {
                right_opt
            } else if let (_, AstNode::NumberLit(1.0)) = (&left_opt, &right_opt) {
                left_opt
            } else if let (AstNode::NumberLit(a), AstNode::NumberLit(b)) = (&left_opt, &right_opt) {
                AstNode::NumberLit(a * b)
            } else {
                AstNode::Mul(Box::new(left_opt), Box::new(right_opt))
            }
        }
        AstNode::Return(expr) => AstNode::Return(expr.map(|e| Box::new(optimize_node(*e)))),
        AstNode::Binary(left, op, right) => {
            let left_opt = optimize_node(*left);
            let right_opt = optimize_node(*right);
            if let (AstNode::NumberLit(a), AstNode::NumberLit(b)) = (&left_opt, &right_opt) {
                match op.as_str() {
                    "+" => AstNode::NumberLit(a + b),
                    "-" => AstNode::NumberLit(a - b),
                    "*" => AstNode::NumberLit(a * b),
                    "/" => AstNode::NumberLit(a / b),
                    "==" => AstNode::BoolLit(a == b),
                    "!=" => AstNode::BoolLit(a != b),
                    "<" => AstNode::BoolLit(a < b),
                    ">" => AstNode::BoolLit(a > b),
                    "<=" => AstNode::BoolLit(a <= b),
                    ">=" => AstNode::BoolLit(a >= b),
                    _ => AstNode::Binary(Box::new(left_opt), op, Box::new(right_opt)),
                }
            } else {
                AstNode::Binary(Box::new(left_opt), op, Box::new(right_opt))
            }
        }
        AstNode::Unary(op, expr) => AstNode::Unary(op, Box::new(optimize_node(*expr))),
        AstNode::Call(name, args) => {
            let opt_args = args.into_iter().map(optimize_node).collect();
            AstNode::Call(name, opt_args)
        }
        other => other,
    }
}

fn is_dead_code(_node: &AstNode) -> bool {
    false
}
