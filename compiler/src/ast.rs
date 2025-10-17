// src/ast.rs - AST definitions

#[derive(Debug, Clone)]
pub enum Ast {
    VarDecl(String, Box<Ast>), // name, value
    Assign(String, Box<Ast>),
    If(Box<Ast>, Vec<Ast>, Option<Vec<Ast>>),
    While(Box<Ast>, Vec<Ast>),
    For(String, Box<Ast>, Box<Ast>, Vec<Ast>), // var, from, to, body
    FuncDef(String, Vec<String>, Vec<Ast>),
    FuncCall(String, Vec<Ast>),
    BinOp(String, Box<Ast>, Box<Ast>),
    Literal(f64),
    StrLit(String),
    Var(String),
    Array(Vec<Ast>),
    Index(String, Box<Ast>), // array name, index
}
