use crate::ast::AstNode;
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

    println!("DEBUG: Parsing code:\n{}", code);

    let pairs = NulaParser::parse(Rule::program, &code)?;
    println!("DEBUG: Parsed pairs count: {}", pairs.len());
    for pair in pairs.clone() {
        println!("DEBUG: Top-level pair rule: {:?}", pair.as_rule());
        for child in pair.into_inner() {
            println!("DEBUG: Child rule: {:?}", child.as_rule());
        }
    }
    Ok(build_ast(pairs))
}

fn build_ast(pairs: Pairs<Rule>) -> AstNode {
    let mut nodes = vec![];
    for pair in pairs {
        println!("DEBUG: Building AST for pair rule: {:?}", pair.as_rule());
        if pair.as_rule() == Rule::COMMENT {
            continue;
        }
        let node = build_node(pair);
        println!("DEBUG: Built node: {:?}", node);
        nodes.push(node);
    }
    println!("DEBUG: Final program nodes count: {}", nodes.len());
    AstNode::Program(nodes)
}

fn build_node(pair: Pair<Rule>) -> AstNode {
    println!("DEBUG: Processing rule in build_node: {:?}", pair.as_rule());
    match pair.as_rule() {
        Rule::translation => {
            let mut inner = pair.into_inner();
            // Skip "#" and "="
            inner.next();
            inner.next();
            let ident = inner
            .next()
            .map(|p| p.as_str().to_string())
            .unwrap_or_default();
            let code_block = inner
            .next()
            .map(|p| p.as_str().to_string())
            .unwrap_or_default();
            let translated = translate_code(&ident, &code_block);
            AstNode::Translation(ident, translated)
        }
        Rule::dependency => AstNode::Dependency(
            pair.into_inner()
            .next()
            .map(|p| p.as_str().to_string())
            .unwrap_or_default(),
        ),
        Rule::import_stmt => AstNode::Import(
            pair.into_inner()
            .next()
            .map(|p| p.as_str().to_string())
            .unwrap_or_default(),
        ),
        Rule::statement => {
            let mut inner = pair.into_inner();
            let stmt = inner.next().unwrap();
            println!("DEBUG: Statement inner child: {:?}", stmt.as_rule());
            build_node(stmt)
        }
        Rule::variable_decl => {
            let mut inner = pair.into_inner();
            let ident = inner
            .next()
            .map(|p| p.as_str().to_string())
            .unwrap_or_default();
            let expr = inner
            .next()
            .map(build_expression)
            .unwrap_or(AstNode::StringLit("".to_string()));
            AstNode::VariableDecl(ident, Box::new(expr))
        }
        Rule::assignment => {
            let mut inner = pair.into_inner();
            // Skip "set"
            inner.next();
            let ident = inner
            .next()
            .map(|p| p.as_str().to_string())
            .unwrap_or_default();
            let expr = inner
            .next()
            .map(build_expression)
            .unwrap_or(AstNode::StringLit("".to_string()));
            AstNode::Assignment(ident, Box::new(expr))
        }
        Rule::function_def => {
            let mut inner = pair.into_inner();
            let name = inner
            .next()
            .map(|p| p.as_str().to_string())
            .unwrap_or_default();
            let params_pair = inner.next();
            let params = params_pair
            .map(|p| p.into_inner().map(|i| i.as_str().to_string()).collect())
            .unwrap_or_default();
            let body_pair = inner.next();
            let body = body_pair
            .map(|p| {
                let statements = p.into_inner();
                statements.map(build_node).collect()
            })
            .unwrap_or_default();
            AstNode::FunctionDef(name, params, body)
        }
        Rule::for_loop => {
            let mut inner = pair.into_inner();
            let var = inner
            .next()
            .map(|p| p.as_str().to_string())
            .unwrap_or_default();
            let iter = inner
            .next()
            .map(build_expression)
            .unwrap_or(AstNode::NumberLit(0.0));
            let body_pair = inner.next().unwrap(); // Block or do..end
            let body = body_pair
            .into_inner()
            .map(build_node)
            .collect();
            AstNode::ForLoop(var, Box::new(iter), body)
        }
        Rule::while_loop => {
            let mut inner = pair.into_inner();
            let cond = inner
            .next()
            .map(build_expression)
            .unwrap_or(AstNode::BoolLit(false));
            let body_pair = inner.next().unwrap(); // Block or do..end
            let body = body_pair
            .into_inner()
            .map(build_node)
            .collect();
            AstNode::WhileLoop(Box::new(cond), body)
        }
        Rule::conditional => {
            let mut inner = pair.into_inner();
            let cond = inner
            .next()
            .map(build_expression)
            .unwrap_or(AstNode::BoolLit(false));
            let body_pair = inner.next().unwrap();
            let body = body_pair
            .into_inner()
            .map(build_node)
            .collect();
            let mut else_ifs = vec![];
            let mut else_body = None;
            for el in inner {
                match el.as_rule() {
                    Rule::else_if => {
                        let mut ei_inner = el.into_inner();
                        let ei_cond = ei_inner
                        .next()
                        .map(build_expression)
                        .unwrap_or(AstNode::BoolLit(false));
                        let ei_body_pair = ei_inner.next().unwrap();
                        let ei_body = ei_body_pair
                        .into_inner()
                        .map(build_node)
                        .collect();
                        else_ifs.push((Box::new(ei_cond), ei_body));
                    }
                    Rule::else_clause => {
                        let el_body_pair = el.into_inner().next().unwrap();
                        let el_body = el_body_pair
                        .into_inner()
                        .map(build_node)
                        .collect();
                        else_body = Some(el_body);
                    }
                    _ => {}
                }
            }
            AstNode::If(Box::new(cond), body, else_ifs, else_body)
        }
        Rule::write_stmt => {
            let mut inner = pair.into_inner();
            println!("DEBUG: Write stmt inner: {:?}", inner.clone().map(|p| p.as_rule()).collect::<Vec<_>>());
            inner.next(); // Skip "write"
            let expr_node = inner.next().map(build_expression).unwrap_or(AstNode::StringLit("".to_string()));
            println!("DEBUG: Write expr: {:?}", expr_node);
            AstNode::Write(Box::new(expr_node))
        }
        Rule::add_stmt => {
            let mut inner = pair.into_inner();
            // Skip "add"
            inner.next();
            let left = inner
            .next()
            .map(build_expression)
            .unwrap_or(AstNode::NumberLit(0.0));
            let right = inner
            .next()
            .map(build_expression)
            .unwrap_or(AstNode::NumberLit(0.0));
            AstNode::Add(Box::new(left), Box::new(right))
        }
        Rule::mul_stmt => {
            let mut inner = pair.into_inner();
            // Skip "mul"
            inner.next();
            let left = inner
            .next()
            .map(build_expression)
            .unwrap_or(AstNode::NumberLit(0.0));
            let right = inner
            .next()
            .map(build_expression)
            .unwrap_or(AstNode::NumberLit(0.0));
            AstNode::Mul(Box::new(left), Box::new(right))
        }
        Rule::return_stmt => AstNode::Return(
            pair.into_inner()
            .next()
            .map(|p| Box::new(build_expression(p))),
        ),
        Rule::expression => {
            println!("DEBUG: Building expression from pair: {:?}", pair.as_str());
            build_expression(pair)
        }
        Rule::COMMENT => AstNode::Comment(pair.as_str().to_string()),
        _ => {
            println!("DEBUG: Unknown rule in build_node: {:?}", pair.as_rule());
            AstNode::Comment(format!("Unknown rule: {:?}", pair.as_rule()))
        }
    }
}

// Funkcje do budowania wyrażeń
fn build_expression(pair: Pair<Rule>) -> AstNode {
    println!("DEBUG: Build expression: {}", pair.as_str());
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
    println!("DEBUG: Build primary: {:?} - {}", pair.as_rule(), pair.as_str());
    match pair.as_rule() {
        Rule::string => {
            let s = pair.as_str().trim_matches(|c| c == '"' || c == '\'').to_string();
            println!("DEBUG: Parsed string: {}", s);
            AstNode::StringLit(s)
        }
        Rule::number => AstNode::NumberLit(pair.as_str().parse::<f64>().unwrap_or(0.0)),
        Rule::bool => AstNode::BoolLit(pair.as_str() == "true"),
        Rule::ident => AstNode::Ident(pair.as_str().to_string()),
        Rule::call => {
            let mut inner = pair.into_inner();
            let name = inner.next().unwrap().as_str().to_string();
            let args_pair = inner.next();
            let args = args_pair
            .map(|arg_list| arg_list.into_inner().map(build_expression).collect())
            .unwrap_or(vec![]);
            AstNode::Call(name, args)
        }
        Rule::paren_expr => build_expression(pair.into_inner().next().unwrap()),
        _ => AstNode::Comment(format!("Unknown primary: {:?}", pair.as_rule())),
    }
}
