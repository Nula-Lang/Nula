use pest::iterators::{Pair, Pairs};

#[derive(Debug)]
pub enum NulaAst {
    Print(String),
    Assign(String, String),  // id = value
    Comment(String),
}

pub fn parse_ast(pairs: Pairs<'_, pest::Rule>) -> Vec<NulaAst> {
    pairs.map(|pair| match pair.as_rule() {
        pest::Rule::print_stmt => {
            let mut inner = pair.into_inner();
            let s = inner.find(|p| p.as_rule() == pest::Rule::string).unwrap().as_str().to_string();
            NulaAst::Print(s.replace("\"", ""))
        }
        pest::Rule::assign_stmt => {
            let mut inner = pair.into_inner();
            let id = inner.next().unwrap().as_str().to_string();
            let val = inner.skip(2).next().unwrap().as_str().to_string();
            NulaAst::Assign(id, val.replace("\"", ""))
        }
        pest::Rule::comment => NulaAst::Comment(pair.as_str().to_string()),
        _ => NulaAst::Comment("unknown".to_string()),
    }).collect()
}
