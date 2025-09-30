use crate::ast::AstNode;
use crate::cli::print_debug;
use crate::translator::translate_code;
use pest::iterators::{Pair, Pairs};
use pest::Parser;
use pest_derive::Parser;
use std::path::Path;

#[derive(Parser)]
#[grammar = "nula.pest"]
pub struct NulaParser;

pub fn parse_nula_file(path: &Path) -> Result<AstNode, pest::error::Error<Rule>> {
    let code = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            return Err(pest::error::Error::new_from_span(
                pest::error::ErrorVariant::CustomError {
                    message: e.to_string(),
                },
                pest::Span::new("", 0, 0).unwrap(),
            ));
        }
    };

    print_debug(&format!("Parsing code:\n{}", code));

    let pairs = NulaParser::parse(Rule::program, &code)?;
    Ok(build_ast(pairs))
}

fn build_ast(pairs: Pairs<Rule>) -> AstNode {
    let mut nodes = vec![];
    for pair in pairs {
        nodes.push(build_node(pair));
    }
    AstNode::Program(nodes)
}

fn build_node(pair: Pair<Rule>) -> AstNode {
    print_debug(&format!("Processing rule: {:?}", pair.as_rule()));
    match pair.as_rule() {
        Rule::translation => {
            let mut inner = pair.into_inner();
            let ident = inner.next().map(|p| p.as_str().to_string()).unwrap_or_default();
            let code_block = inner.next().map(|p| p.as_str().to_string()).unwrap_or_default();
            let translated = translate_code(&ident, &code_block);
            AstNode::Translation(ident, translated)
        }
        Rule::dependency => AstNode::Dependency(pair.into_inner().next().map(|p| p.as_str().to_string()).unwrap_or_default()),
        Rule::import_stmt => AstNode::Import(pair.into_inner().next().map(|p| p.as_str().to_string()).unwrap_or_default()),
        Rule::variable_decl => {
            let mut inner = pair.into_inner();
            let ident = inner.next().map(|p| p.as_str().to_string()).unwrap_or_default();
            let expr = inner.next().map(build_expression).unwrap_or(AstNode::NumberLit(0.0));
            AstNode::VariableDecl(ident, Box::new(expr))
        }
        Rule::assignment => {
            let mut inner = pair.into_inner();
            let ident = inner.next().map(|p| p.as_str().to_string()).unwrap_or_default();
            let expr = inner.next().map(build_expression).unwrap_or(AstNode::NumberLit(0.0));
            AstNode::Assignment(ident, Box::new(expr))
        }
        Rule::function_def => {
            let mut inner = pair.into_inner();
            let name = inner.next().map(|p| p.as_str().to_string()).unwrap_or_default();
            let params = inner.next().map(|p| p.into_inner().map(|i| i.as_str().to_string()).collect()).unwrap_or_default();
            let body = inner.next().map(|p| p.into_inner().map(build_node).collect()).unwrap_or_default();
            AstNode::FunctionDef(name, params, body)
        }
        Rule::for_loop => {
            let mut inner = pair.into_inner();
            let var = inner.next().map(|p| p.as_str().to_string()).unwrap_or_default();
            let iter = inner.next().map(build_expression).unwrap_or(AstNode::NumberLit(0.0));
            let body = inner.next().map(|p| p.into_inner().map(build_node).collect()).unwrap_or_default();
            AstNode::ForLoop(var, Box::new(iter), body)
        }
        Rule::while_loop => {
            let mut inner = pair.into_inner();
            let cond = inner.next().map(build_expression).unwrap_or(AstNode::BoolLit(false));
            let block = inner.next().unwrap();
            let body = block.into_inner().map(build_node).collect::<Vec<AstNode>>();
            AstNode::WhileLoop(Box::new(cond), body)
        }
        Rule::conditional => {
            let mut inner = pair.into_inner();
            let cond = inner.next().map(build_expression).unwrap_or(AstNode::BoolLit(false));
            let block = inner.next().unwrap();
            let body = block.into_inner().map(build_node).collect::<Vec<AstNode>>();
            let mut else_ifs = vec![];
            let mut else_body = None;
            for el in inner {
                if el.as_rule() == Rule::else_if {
                    let mut ei_inner = el.into_inner();
                    let ei_cond = ei_inner.next().map(build_expression).unwrap_or(AstNode::BoolLit(false));
                    let ei_block = ei_inner.next().unwrap();
                    let ei_body = ei_block.into_inner().map(build_node).collect::<Vec<AstNode>>();
                    else_ifs.push((Box::new(ei_cond), ei_body));
                } else if el.as_rule() == Rule::else_clause {
                    let el_block = el.into_inner().next().unwrap();
                    else_body = Some(el_block.into_inner().map(build_node).collect::<Vec<AstNode>>());
                }
            }
            AstNode::If(Box::new(cond), body, else_ifs, else_body)
        }
        Rule::write_stmt => AstNode::Write(Box::new(pair.into_inner().next().map(build_expression).unwrap_or(AstNode::NumberLit(0.0)))),
        Rule::add_stmt => {
            let mut inner = pair.into_inner();
            let left = inner.next().map(build_expression).unwrap_or(AstNode::NumberLit(0.0));
            let right = inner.next().map(build_expression).unwrap_or(AstNode::NumberLit(0.0));
            AstNode::Add(Box::new(left), Box::new(right))
        }
        Rule::mul_stmt => {
            let mut inner = pair.into_inner();
            let left = inner.next().map(build_expression).unwrap_or(AstNode::NumberLit(0.0));
            let right = inner.next().map(build_expression).unwrap_or(AstNode::NumberLit(0.0));
            AstNode::Mul(Box::new(left), Box::new(right))
        }
        Rule::return_stmt => AstNode::Return(pair.into_inner().next().map(|p| Box::new(build_expression(p)))),
        Rule::expression => build_expression(pair.into_inner().next().unwrap()),
        _ => AstNode::Comment(pair.as_str().to_string()),
    }
}

fn build_expression(pair: Pair<Rule>) -> AstNode {
    let mut inner = pair.into_inner();
    let mut left = build_logic_expr(inner.next().unwrap());
    while let Some(op_pair) = inner.next() {
        let op = op_pair.as_str().to_string();
        let right = build_logic_expr(inner.next().unwrap());
        left = AstNode::Binary(Box::new(left), op, Box::new(right));
    }
    left
}

fn build_logic_expr(pair: Pair<Rule>) -> AstNode {
    let mut inner = pair.into_inner();
    let mut left = build_compare_expr(inner.next().unwrap());
    while let Some(op_pair) = inner.next() {
        let op = op_pair.as_str().to_string();
        let right = build_compare_expr(inner.next().unwrap());
        left = AstNode::Binary(Box::new(left), op, Box::new(right));
    }
    left
}

fn build_compare_expr(pair: Pair<Rule>) -> AstNode {
    let mut inner = pair.into_inner();
    let mut left = build_add_expr(inner.next().unwrap());
    while let Some(op_pair) = inner.next() {
        let op = op_pair.as_str().to_string();
        let right = build_add_expr(inner.next().unwrap());
        left = AstNode::Binary(Box::new(left), op, Box::new(right));
    }
    left
}

fn build_add_expr(pair: Pair<Rule>) -> AstNode {
    let mut inner = pair.into_inner();
    let mut left = build_mul_expr(inner.next().unwrap());
    while let Some(op_pair) = inner.next() {
        let op = op_pair.as_str().to_string();
        let right = build_mul_expr(inner.next().unwrap());
        left = AstNode::Binary(Box::new(left), op, Box::new(right));
    }
    left
}

fn build_mul_expr(pair: Pair<Rule>) -> AstNode {
    let mut inner = pair.into_inner();
    let mut left = build_unary_expr(inner.next().unwrap());
    while let Some(op_pair) = inner.next() {
        let op = op_pair.as_str().to_string();
        let right = build_unary_expr(inner.next().unwrap());
        left = AstNode::Binary(Box::new(left), op, Box::new(right));
    }
    left
}

fn build_unary_expr(pair: Pair<Rule>) -> AstNode {
    let mut inner = pair.into_inner();
    let first = inner.next().unwrap();
    if first.as_rule() == Rule::unary_op {
        let op = first.as_str().to_string();
        let expr = build_primary(inner.next().unwrap());
        AstNode::Unary(op, Box::new(expr))
    } else {
        build_primary(first)
    }
}

fn build_primary(pair: Pair<Rule>) -> AstNode {
    match pair.as_rule() {
        Rule::string => AstNode::StringLit(pair.as_str().trim_matches(|c| c == '"' || c == '\'').to_string()),
        Rule::number => AstNode::NumberLit(pair.as_str().parse::<f64>().unwrap_or(0.0)),
        Rule::bool => AstNode::BoolLit(pair.as_str() == "true"),
        Rule::ident => AstNode::Ident(pair.as_str().to_string()),
        Rule::call => {
            let mut inner = pair.into_inner();
            let name = inner.next().unwrap().as_str().to_string();
            let args = inner.next().map(|arg_list| arg_list.into_inner().map(build_expression).collect()).unwrap_or(vec![]);
            AstNode::Call(name, args)
        }
        Rule::paren_expr => build_expression(pair.into_inner().next().unwrap()),
        _ => AstNode::Comment(pair.as_str().to_string()),
    }
}
