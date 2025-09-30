use crate::cli::{print_debug, print_error};
use crate::process_expression::process_expression;
use crate::translator::translate_code;
use pest::Parser;
use pest_derive::Parser;
use std::path::Path;

#[derive(Parser)]
#[grammar = "nula.pest"]
pub struct NulaParser;

pub fn parse_nula_file(path: &Path) -> Result<String, pest::error::Error<Rule>> {
    let code = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            print_error(&format!("Failed to read file {:?}: {}", path, e));
            return Err(pest::error::Error::new_from_span(
                pest::error::ErrorVariant::CustomError {
                    message: e.to_string(),
                },
                pest::Span::new("", 0, 0).unwrap(),
            ));
        }
    };

    print_debug(&format!("Parsing code:\n{}", code));

    let pairs = match NulaParser::parse(Rule::program, &code) {
        Ok(p) => p,
        Err(e) => {
            print_error(&format!("Parsing failed: {}", e));
            return Err(e);
        }
    };

    let mut ast = String::new();
    process_pairs(pairs, &mut ast);
    Ok(ast)
}

fn process_pairs(pairs: pest::iterators::Pairs<Rule>, ast: &mut String) {
    for pair in pairs {
        process_pair(pair, ast);
    }
}

fn process_pair(pair: pest::iterators::Pair<Rule>, ast: &mut String) {
    print_debug(&format!("Processing rule: {:?}", pair.as_rule()));
    match pair.as_rule() {
        Rule::translation => {
            let mut inner = pair.into_inner();
            let lang = inner.next().map(|p| p.as_str().trim()).unwrap_or("");
            let code_block = inner.next().map(|p| p.as_str()).unwrap_or("");
            let translated = translate_code(lang, code_block);
            ast.push_str(&translated);
            ast.push('\n');
        }
        Rule::dependency => {
            let dep = pair.into_inner().next().map(|p| p.as_str().trim()).unwrap_or("");
            ast.push_str(&format!("// Resolved dependency: {}\n", dep));
        }
        Rule::import_stmt => {
            let import = pair.into_inner().next().map(|p| p.as_str().trim()).unwrap_or("");
            ast.push_str(&format!("import {}\n", import));
        }
        Rule::statement => {
            process_pairs(pair.into_inner(), ast);
        }
        Rule::variable_decl => {
            let mut inner = pair.into_inner();
            let name = inner.next().map(|p| p.as_str()).unwrap_or("");
            let value = process_expression(inner.next().unwrap());
            ast.push_str(&format!("var {} = {}\n", name, value));
        }
        Rule::function_def => {
            let mut inner = pair.into_inner();
            let name = inner.next().map(|p| p.as_str()).unwrap_or("");
            let params = inner.next().map(|p| {
                p.into_inner()
                .map(|i| i.as_str())
                .collect::<Vec<_>>()
                .join(", ")
            }).unwrap_or_default();
            ast.push_str(&format!("fn {}({}) {{\n", name, params));
            process_pairs(inner, ast);
            ast.push_str("}\n");
        }
        Rule::loop_stmt | Rule::for_loop | Rule::while_loop => {
            ast.push_str(pair.as_str());
            ast.push('\n');
        }
        Rule::conditional => {
            ast.push_str(pair.as_str());
            ast.push('\n');
        }
        Rule::write_stmt => {
            let expr = process_expression(pair.into_inner().next().unwrap());
            ast.push_str(&format!("write {}\n", expr));
        }
        Rule::add_stmt => {
            let mut inner = pair.into_inner();
            let a = process_expression(inner.next().unwrap());
            let b = process_expression(inner.next().unwrap());
            ast.push_str(&format!("add {} {}\n", a, b));
        }
        Rule::mul_stmt => {
            let mut inner = pair.into_inner();
            let a = process_expression(inner.next().unwrap());
            let b = process_expression(inner.next().unwrap());
            ast.push_str(&format!("mul {} {}\n", a, b));
        }
        Rule::return_stmt => {
            let expr = pair.into_inner().next().map(|p| process_expression(p)).unwrap_or_default();
            ast.push_str(&format!("return {}\n", expr));
        }
        Rule::expression => {
            let expr_str = process_expression(pair);
            ast.push_str(&expr_str);
        }
        _ => {}
    }
}
