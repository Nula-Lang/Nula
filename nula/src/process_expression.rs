use pest::iterators::Pair;
use super::parser::Rule;

pub fn process_expression(pair: Pair<Rule>) -> String {
    match pair.as_rule() {
        Rule::expression | Rule::logic_expr | Rule::compare_expr | Rule::add_expr | Rule::mul_expr => {
            let mut inner = pair.into_inner();
            let mut expr = process_expression(inner.next().unwrap());
            while let Some(op) = inner.next() {
                let rhs = process_expression(inner.next().unwrap());
                expr = format!("{} {} {}", expr, op.as_str(), rhs);
            }
            expr
        }
        Rule::unary_expr => {
            let mut inner = pair.into_inner();
            if inner.clone().count() == 2 {
                let op = inner.next().unwrap().as_str();
                let primary = process_expression(inner.next().unwrap());
                format!("{}{}", op, primary)
            } else {
                process_expression(inner.next().unwrap())
            }
        }
        Rule::primary => process_expression(pair.into_inner().next().unwrap()),
        Rule::atom => pair.as_str().to_string(),
        Rule::paren_expr => {
            format!("({})", process_expression(pair.into_inner().next().unwrap()))
        }
        Rule::call => {
            let mut inner = pair.into_inner();
            let name = inner.next().unwrap().as_str();
            let args = inner.next().map(|p| {
                p.into_inner()
                .map(|a| process_expression(a))
                .collect::<Vec<_>>()
                .join(", ")
            }).unwrap_or_default();
            format!("{}({})", name, args)
        }
        Rule::string => format!("\"{}\"", pair.as_str().trim_matches('"')),
        Rule::number | Rule::bool | Rule::ident => pair.as_str().to_string(),
        _ => pair.as_str().to_string(),
    }
}
