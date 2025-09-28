use crate::ast::{AstNode, Expr};
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "../nula.pest"]
pub struct NulaParser;

pub fn parse_to_ast(pairs: pest::iterators::Pairs<Rule>) -> Vec<AstNode> {
    let mut ast = Vec::new();
    for pair in pairs {
        match pair.as_rule() {
            Rule::write_stmt => {
                let inner = pair.into_inner().next().unwrap();
                ast.push(AstNode::Write(inner.as_str().to_string()));
            }
            Rule::comment => ast.push(AstNode::Comment(pair.as_str().to_string())),
            Rule::dependency => ast.push(AstNode::Dependency(pair.into_inner().next().unwrap().as_str().to_string())),
            Rule::foreign_block => {
                let mut inner = pair.into_inner();
                let lang = inner.next().unwrap().as_str().to_string();
                let code = inner.next().unwrap().as_str().to_string();
                ast.push(AstNode::ForeignBlock { lang, code });
            }
            Rule::let_stmt => {
                let mut inner = pair.into_inner();
                let name = inner.next().unwrap().as_str().to_string();
                let value = parse_expr(inner.next().unwrap());
                ast.push(AstNode::Let { name, value });
            }
            Rule::for_stmt => {
                let mut inner = pair.into_inner();
                let var = inner.next().unwrap().as_str().to_string();
                let range_pair = inner.next().unwrap().into_inner();
                let start = range_pair.clone().next().unwrap().as_str().parse::<i32>().unwrap();
                let end = range_pair.next().unwrap().as_str().parse::<i32>().unwrap();
                let body = parse_to_ast(inner.next().unwrap().into_inner());
                ast.push(AstNode::For { var, range: (start, end), body });
            }
            Rule::if_stmt => {
                let mut inner = pair.into_inner();
                let cond = parse_expr(inner.next().unwrap());
                let body = parse_to_ast(inner.next().unwrap().into_inner());
                ast.push(AstNode::If { cond, body });
            }
            Rule::fn_def => {
                let mut inner = pair.into_inner();
                let name = inner.next().unwrap().as_str().to_string();
                let params_pair = inner.next().unwrap().into_inner();
                let mut params = Vec::new();
                for p in params_pair {
                    params.push(p.as_str().to_string());
                }
                let body = parse_to_ast(inner.next().unwrap().into_inner());
                let ret = Box::new(parse_expr(inner.next().unwrap()));
                ast.push(AstNode::Fn { name, params, body, ret });
            }
            _ => {}
        }
    }
    ast
}

fn parse_expr(pair: Pair<Rule>) -> Expr {
    match pair.as_rule() {
        Rule::int => Expr::Int(pair.as_str().parse().unwrap()),
        Rule::string => Expr::Str(pair.as_str().to_string()),
        Rule::var => Expr::Var(pair.as_str().to_string()),
        Rule::bin_op => {
            let mut inner = pair.into_inner();
            let left = Box::new(parse_expr(inner.next().unwrap()));
            let op = inner.next().unwrap().as_str().chars().next().unwrap();
            let right = Box::new(parse_expr(inner.next().unwrap()));
            Expr::BinOp { op, left, right }
        }
        Rule::call => {
            let mut inner = pair.into_inner();
            let name = inner.next().unwrap().as_str().to_string();
            let args_pair = inner.next().unwrap().into_inner();
            let mut args = Vec::new();
            for a in args_pair {
                args.push(parse_expr(a));
            }
            Expr::Call { name, args }
        }
        _ => unreachable!(),
    }
}
