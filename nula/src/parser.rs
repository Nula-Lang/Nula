use crate::ast::{AstNode, Expr, Type, AstTree, AstError, FnType};
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "../nula.pest"]
pub struct NulaParser;

pub fn parse_to_ast(pairs: pest::iterators::Pairs<Rule>) -> Result<AstTree, AstError> {
    let mut tree = AstTree::new();
    for pair in pairs {
        let node = parse_node(pair)?;
        tree.add_node(tree.root, node);
    }
    Ok(tree)
}

fn parse_node(pair: Pair<Rule>) -> Result<AstNode, AstError> {
    match pair.as_rule() {
        Rule::write_stmt => Ok(AstNode::Write(parse_expr(pair.into_inner().next().unwrap())?)),
        Rule::comment => Ok(AstNode::Comment(pair.as_str().to_string())),
        Rule::dependency => Ok(AstNode::Dependency(pair.into_inner().next().unwrap().as_str().to_string())),
        Rule::foreign_block => {
            let mut inner = pair.into_inner();
            let lang = inner.next().unwrap().as_str().to_string();
            let code = inner.next().unwrap().as_str().to_string();
            Ok(AstNode::ForeignBlock { lang, code })
        }
        Rule::let_stmt => {
            let mut inner = pair.into_inner();
            let name = inner.next().unwrap().as_str().to_string();
            let ty = inner.next().map(parse_type).transpose()?;
            let value = parse_expr(inner.next().unwrap())?;
            Ok(AstNode::Let { name, ty, value })
        }
        Rule::for_stmt => {
            let mut inner = pair.into_inner();
            let var = inner.next().unwrap().as_str().to_string();
            let ty = inner.next().map(parse_type).transpose()?;
            let start = parse_expr(inner.next().unwrap())?;
            let end = parse_expr(inner.next().unwrap())?;
            let body = parse_body(inner.next().unwrap())?;
            Ok(AstNode::For { var, ty, range: (start, end), body })
        }
        Rule::if_stmt => {
            let mut inner = pair.into_inner();
            let cond = parse_expr(inner.next().unwrap())?;
            let body = parse_body(inner.next().unwrap())?;
            let else_body = if inner.peek().is_some() { Some(parse_body(inner.next().unwrap())?) } else { None };
            Ok(AstNode::If { cond, body, else_body })
        }
        Rule::while_stmt => {
            let mut inner = pair.into_inner();
            let cond = parse_expr(inner.next().unwrap())?;
            let body = parse_body(inner.next().unwrap())?;
            Ok(AstNode::While { cond, body })
        }
        Rule::fn_def => {
            let mut inner = pair.into_inner();
            let name = inner.next().unwrap().as_str().to_string();
            let generics = parse_generics(inner.next().unwrap())?;
            let params = parse_params(inner.next().unwrap())?;
            let ret_ty = parse_type(inner.next().unwrap())?;
            let body = parse_body(inner.next().unwrap())?;
            Ok(AstNode::Fn { name, generics, params, ret_ty, body })
        }
        Rule::struct_def => {
            let mut inner = pair.into_inner();
            let name = inner.next().unwrap().as_str().to_string();
            let generics = parse_generics(inner.next().unwrap())?;
            let fields = parse_fields(inner.next().unwrap())?;
            let impl_traits = parse_impl_traits(inner.next())?;
            Ok(AstNode::StructDef { name, generics, fields, impl_traits })
        }
        Rule::enum_def => {
            let mut inner = pair.into_inner();
            let name = inner.next().unwrap().as_str().to_string();
            let generics = parse_generics(inner.next().unwrap())?;
            let variants = parse_variants(inner.next().unwrap())?;
            Ok(AstNode::EnumDef { name, generics, variants })
        }
        Rule::trait_def => {
            let mut inner = pair.into_inner();
            let name = inner.next().unwrap().as_str().to_string();
            let methods = parse_trait_methods(inner.next().unwrap())?;
            Ok(AstNode::TraitDef { name, methods })
        }
        Rule::impl_block => {
            let mut inner = pair.into_inner();
            let trait_name = inner.next().map(|p| p.as_str().to_string());
            let ty = parse_type(inner.next().unwrap())?;
            let methods = parse_body(inner.next().unwrap())?;  // Expect Fn nodes
            Ok(AstNode::Impl { ty, trait_name, methods })
        }
        Rule::class_def => {
            let mut inner = pair.into_inner();
            let name = inner.next().unwrap().as_str().to_string();
            let generics = parse_generics(inner.next().unwrap())?;
            let fields = parse_fields(inner.next().unwrap())?;
            let methods = parse_body(inner.next().unwrap())?;
            Ok(AstNode::ClassDef { name, generics, fields, methods })
        }
        _ => Ok(AstNode::Expr(parse_expr(pair)?)),
    }
}

fn parse_expr(pair: Pair<Rule>) -> Result<Expr, AstError> {
    match pair.as_rule() {
        Rule::int => Ok(Expr::Int(pair.as_str().parse().unwrap())),
        Rule::float => Ok(Expr::Float(pair.as_str().parse().unwrap())),
        Rule::string => Ok(Expr::Str(pair.as_str().to_string())),
        Rule::bool => Ok(Expr::Bool(pair.as_str() == "true")),
        Rule::var => {
            let mut inner = pair.into_inner();
            let name = inner.next().unwrap().as_str().to_string();
            let generics = if inner.peek().is_some() { inner.next().unwrap().into_inner().map(parse_type).collect::<Result<_, _>>()? } else { vec![] };
            Ok(Expr::Var { name, generics })
        }
        Rule::bin_op => {
            let mut inner = pair.into_inner();
            let left = Box::new(parse_expr(inner.next().unwrap())?);
            let op = inner.next().unwrap().as_str().to_string();
            let right = Box::new(parse_expr(inner.next().unwrap())?);
            Ok(Expr::BinOp { op, left, right })
        }
        Rule::unary_op => {
            let mut inner = pair.into_inner();
            let op = inner.next().unwrap().as_str().to_string();
            let expr = Box::new(parse_expr(inner.next().unwrap())?);
            Ok(Expr::UnaryOp { op, expr })
        }
        Rule::call => {
            let mut inner = pair.into_inner();
            let callee = Box::new(parse_expr(inner.next().unwrap())?);
            let args = inner.next().unwrap().into_inner().map(parse_expr).collect::<Result<_, _>>()?;
            Ok(Expr::Call { callee, args })
        }
        Rule::array => Ok(Expr::Array(pair.into_inner().map(parse_expr).collect::<Result<_, _>>()?)),
        Rule::index => {
            let mut inner = pair.into_inner();
            let arr = Box::new(parse_expr(inner.next().unwrap())?);
            let idx = Box::new(parse_expr(inner.next().unwrap())?);
            Ok(Expr::Index { arr, idx })
        }
        Rule::length => Ok(Expr::Length(Box::new(parse_expr(pair.into_inner().next().unwrap())?))),
        Rule::struct_lit => {
            let mut inner = pair.into_inner();
            let ty = parse_type(inner.next().unwrap())?;
            let fields = inner.map(|f| {
                let mut fi = f.into_inner();
                let name = fi.next().unwrap().as_str().to_string();
                let value = parse_expr(fi.next().unwrap())?;
                Ok((name, value))
            }).collect::<Result<_, _>>()?;
            Ok(Expr::StructLit { ty, fields })
        }
        Rule::enum_lit => {
            let mut inner = pair.into_inner();
            let ty = parse_type(inner.next().unwrap())?;
            let variant = inner.next().unwrap().as_str().to_string();
            let value = inner.next().map(|p| Box::new(parse_expr(p)?));
            Ok(Expr::EnumLit { ty, variant, value })
        }
        Rule::field_access => {
            let mut inner = pair.into_inner();
            let obj = Box::new(parse_expr(inner.next().unwrap())?);
            let field = inner.next().unwrap().as_str().to_string();
            Ok(Expr::FieldAccess { obj, field })
        }
        Rule::method_call => {
            let mut inner = pair.into_inner();
            let obj = Box::new(parse_expr(inner.next().unwrap())?);
            let method = inner.next().unwrap().as_str().to_string();
            let args = inner.next().unwrap().into_inner().map(parse_expr).collect::<Result<_, _>>()?;
            Ok(Expr::MethodCall { obj, method, args })
        }
        _ => Err(AstError::Undefined("Unknown expr".to_string())),
    }
}

fn parse_type(pair: Pair<Rule>) -> Result<Type, AstError> {
    match pair.as_rule() {
        Rule::type_int => Ok(Type::Int),
        Rule::type_str => Ok(Type::Str),
        Rule::type_bool => Ok(Type::Bool),
        Rule::type_float => Ok(Type::Float),
        Rule::type_array => Ok(Type::Array(Box::new(parse_type(pair.into_inner().next().unwrap())?))),
        Rule::type_struct => {
            let mut inner = pair.into_inner();
            let name = inner.next().unwrap().as_str().to_string();
            let generics = inner.map(parse_type).collect::<Result<_, _>>()?;
            Ok(Type::Struct { name, fields: vec![], generics })  // Fields resolved later
        }
        Rule::type_enum => {
            let mut inner = pair.into_inner();
            let name = inner.next().unwrap().as_str().to_string();
            let generics = inner.map(parse_type).collect::<Result<_, _>>()?;
            Ok(Type::Enum { name, variants: vec![], generics })
        }
        Rule::type_trait => Ok(Type::Trait { name: pair.as_str().to_string(), methods: vec![] }),
        Rule::generic_param => Ok(Type::Generic(pair.as_str().to_string())),
        _ => Err(AstError::Undefined("Unknown type".to_string())),
    }
}

fn parse_generics(pair: Pair<Rule>) -> Result<Vec<String>, AstError> {
    Ok(pair.into_inner().map(|p| p.as_str().to_string()).collect())
}

fn parse_params(pair: Pair<Rule>) -> Result<Vec<(String, Type)>, AstError> {
    pair.into_inner().map(|p| {
        let mut pi = p.into_inner();
        let name = pi.next().unwrap().as_str().to_string();
        let ty = parse_type(pi.next().unwrap())?;
        Ok((name, ty))
    }).collect()
}

fn parse_fields(pair: Pair<Rule>) -> Result<Vec<(String, Type)>, AstError> {
    parse_params(pair)  // Similar
}

fn parse_variants(pair: Pair<Rule>) -> Result<Vec<(String, Option<Type>)>, AstError> {
    pair.into_inner().map(|p| {
        let mut pi = p.into_inner();
        let name = pi.next().unwrap().as_str().to_string();
        let ty = pi.next().map(parse_type).transpose()?;
        Ok((name, ty))
    }).collect()
}

fn parse_trait_methods(pair: Pair<Rule>) -> Result<Vec<(String, FnType)>, AstError> {
    pair.into_inner().map(|p| {
        let mut pi = p.into_inner();
        let name = pi.next().unwrap().as_str().to_string();
        let params = parse_params(pi.next().unwrap())?.into_iter().map(|(_, t)| t).collect();
        let ret = Box::new(parse_type(pi.next().unwrap())?);
        Ok((name, FnType { params, ret }))
    }).collect()
}

fn parse_impl_traits(pair: Option<Pair<Rule>>) -> Vec<String> {
    pair.map(|p| p.into_inner().map(|i| i.as_str().to_string()).collect()).unwrap_or(vec![])
}

fn parse_body(pair: Pair<Rule>) -> Result<Vec<AstNode>, AstError> {
    pair.into_inner().map(parse_node).collect()
}
