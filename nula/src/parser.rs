use crate::ast::{AstNode, Expr, AstTree, Type};
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "../nula.pest"]
pub struct NulaParser;

pub fn parse_to_ast(pairs: pest::iterators::Pairs<Rule>) -> AstTree {
    let mut tree = AstTree::new();
    for pair in pairs {
        let node = parse_node(pair);
        tree.root.append(tree.arena.new_node(node), &mut tree.arena);
    }
    tree.infer_types();
    tree
}

fn parse_node(pair: Pair<Rule>) -> AstNode {
    match pair.as_rule() {
        Rule::write_stmt => AstNode::Write(parse_expr(pair.into_inner().next().unwrap())),
        Rule::comment => AstNode::Comment(pair.as_str().to_string()),
        Rule::dependency => AstNode::Dependency(pair.into_inner().next().unwrap().as_str().to_string()),
        Rule::foreign_block => {
            let mut inner = pair.into_inner();
            let lang = inner.next().unwrap().as_str().to_string();
            let code = inner.next().unwrap().as_str().to_string();
            AstNode::ForeignBlock { lang, code }
        }
        Rule::let_stmt => {
            let mut inner = pair.into_inner();
            let name = inner.next().unwrap().as_str().to_string();
            let value = parse_expr(inner.next().unwrap());
            let ty = infer_type(&value);
            AstNode::Let { name, value, ty }
        }
        Rule::for_stmt => {
            let mut inner = pair.into_inner();
            let var = inner.next().unwrap().as_str().to_string();
            let start = parse_expr(inner.next().unwrap());
            let end = parse_expr(inner.next().unwrap());
            let body = inner.next().unwrap().into_inner().map(parse_node).collect();
            AstNode::For { var, range: (start, end), body }
        }
        Rule::if_stmt => {
            let mut inner = pair.into_inner();
            let cond = parse_expr(inner.next().unwrap());
            let body = inner.next().unwrap().into_inner().map(parse_node).collect();
            AstNode::If { cond, body }
        }
        Rule::while_stmt => {
            let mut inner = pair.into_inner();
            let cond = parse_expr(inner.next().unwrap());
            let body = inner.next().unwrap().into_inner().map(parse_node).collect();
            AstNode::While { cond, body }
        }
        Rule::fn_def => {
            let mut inner = pair.into_inner();
            let name = inner.next().unwrap().as_str().to_string();
            let params_pair = inner.next().unwrap().into_inner();
            let mut params = Vec::new();
            for p in params_pair {
                params.push((p.as_str().to_string(), Type::Unknown));
            }
            let body = inner.next().unwrap().into_inner().map(parse_node).collect();
            let ret = Box::new(parse_expr(inner.next().unwrap()));
            let ret_ty = infer_type(&ret);
            AstNode::Fn { name, params, body, ret, ret_ty }
        }
        _ => AstNode::Expr(parse_expr(pair)),
    }
}

pub fn parse_expr(pair: Pair<Rule>) -> Expr {
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
            let args = inner.next().unwrap().into_inner().map(parse_expr).collect();
            Expr::Call { name, args }
        }
        Rule::array => {
            let elements = pair.into_inner().map(parse_expr).collect();
            Expr::Array(elements)
        }
        Rule::index => {
            let mut inner = pair.into_inner();
            let arr = Box::new(parse_expr(inner.next().unwrap()));
            let idx = Box::new(parse_expr(inner.next().unwrap()));
            Expr::Index { arr, idx }
        }
        Rule::length => Expr::Length(Box::new(parse_expr(pair.into_inner().next().unwrap()))),
        _ => unreachable!(),
    }
}

fn infer_type(expr: &Expr) -> Type {
    match expr {
        Expr::Int(_) => Type::Int,
        Expr::Str(_) => Type::Str,
        Expr::Array(elems) if !elems.is_empty() => Type::Array(Box::new(infer_type(&elems[0]))),
        _ => Type::Unknown,
    }
}
