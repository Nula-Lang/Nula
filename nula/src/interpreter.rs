use crate::ast::{AstTree, Expr, AstNode, Type};
use std::collections::HashMap;

#[derive(Clone)]
enum Value {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Array(Vec<Value>),
    Struct { fields: HashMap<String, Value> },
    Enum { variant: String, value: Option<Box<Value>> },
    Fn { params: Vec<String>, body: Vec<AstNode> },  // Closure stub
}

pub fn interpret_ast(ast: &AstTree) -> Result<(), AstError> {
    let mut globals = HashMap::new();
    for child in ast.root.children(&ast.arena) {
        interpret_node(ast, &ast.arena.get(child).unwrap().get(), &mut globals)?;
    }
    Ok(())
}

fn interpret_node(ast: &AstTree, node: &AstNode, env: &mut HashMap<String, Value>) -> Result<Value, AstError> {
    match node {
        AstNode::Write(e) => {
            let val = interpret_expr(e, env)?;
            println!("{:?}", val);
            Ok(Value::Int(0))
        }
        AstNode::Let { name, value, .. } => {
            let val = interpret_expr(value, env)?;
            env.insert(name.clone(), val);
            Ok(Value::Int(0))
        }
        // ... Poważna interpretacja dla for (loop), if, while, fn (def closure), call (exec), struct/enum lit, method (dispatch based on type)
        _ => Ok(Value::Int(0)),
    }
}

fn interpret_expr(expr: &Expr, env: &HashMap<String, Value>) -> Result<Value, AstError> {
    match expr {
        Expr::Int(i) => Ok(Value::Int(*i)),
        Expr::Float(f) => Ok(Value::Float(*f)),
        // ... Dla binop (match types), call, etc.
        Expr::MethodCall { obj, method, args } => {
            let obj_val = interpret_expr(obj, env)?;
            // Dynamic dispatch: find method in obj type
            Ok(Value::Int(0))
        }
        _ => Err(AstError::Undefined("Unsupported".to_string())),
    }
}
