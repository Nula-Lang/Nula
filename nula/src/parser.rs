use crate::cli::{print_debug, print_error};
use crate::translator::translate_code;
use pest::iterators::{Pair, Pairs};
use pest::pratt_parser::{Assoc, Op, PrattParser};
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

fn process_pairs(pairs: Pairs<Rule>, ast: &mut String) {
    for pair in pairs {
        process_pair(pair, ast);
    }
}

fn process_pair(pair: Pair<Rule>, ast: &mut String) {
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
        Rule::statement => {
            ast.push_str(pair.as_str().trim());
            ast.push('\n');
        }
        Rule::variable_decl => {
            let mut inner = pair.into_inner();
            let name = inner.next().map(|p| p.as_str()).unwrap_or("");
            let value = inner.next().map(|p| p.as_str()).unwrap_or("");
            ast.push_str(&format!("var {} = {}\n", name, value));
        }
        Rule::function_def => {
            ast.push_str("fn ");
            process_pairs(pair.into_inner(), ast);
            ast.push('\n');
        }
        Rule::loop_stmt => {
            ast.push_str(pair.as_str());
            ast.push('\n');
        }
        Rule::conditional => {
            ast.push_str(pair.as_str());
            ast.push('\n');
        }
        Rule::write_stmt => {
            ast.push_str("write ");
            process_pairs(pair.into_inner(), ast);
            ast.push('\n');
        }
        Rule::add_stmt => {
            ast.push_str("add ");
            process_pairs(pair.into_inner(), ast);
            ast.push('\n');
        }
        Rule::expression => {
            let pratt = PrattParser::new()
            .op(Op::infix(Rule::bin_op, Assoc::Left))
            .op(Op::prefix(Rule::unary_expr));
            let expr_str = pratt
            .map_primary(|primary| match primary.as_rule() {
                Rule::atom => primary.as_str().to_string(),
                         Rule::call => {
                             let mut call_str = String::new();
                             for inner in primary.into_inner() {
                                 match inner.as_rule() {
                                     Rule::ident => call_str.push_str(inner.as_str()),
                         Rule::expression => call_str.push_str(&format!("({})", inner.as_str())),
                         _ => call_str.push_str(inner.as_str()),
                                 }
                             }
                             call_str
                         }
                         Rule::paren_expr => format!("({})", primary.as_str()),
                         _ => primary.as_str().to_string(),
            })
            .map_infix(|lhs, op, rhs| {
                format!("{} {} {}", lhs, op.as_str(), rhs)
            })
            .map_prefix(|op, rhs| {
                format!("{}{}", op.as_str(), rhs)
            })
            .parse(pair.into_inner());
            ast.push_str(&expr_str);
        }
        Rule::binary_expr => {
            process_pairs(pair.into_inner(), ast);
        }
        Rule::call => {
            process_pairs(pair.into_inner(), ast);
        }
        _ => {
            ast.push_str(pair.as_str());
        }
    }
}
