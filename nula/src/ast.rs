#[derive(Debug, Clone)]
pub enum AstNode {
    Program(Vec<AstNode>),
    Translation(String, String), // lang, code
    Dependency(String), // Supports both ident and file_path
    Import(String),     // Supports both ident and file_path
    Comment(String),
    VariableDecl(String, Box<AstNode>),
    Assignment(String, Box<AstNode>),
    FunctionDef(String, Vec<String>, Vec<AstNode>),
    ForLoop(String, Box<AstNode>, Vec<AstNode>),
    WhileLoop(Box<AstNode>, Vec<AstNode>),
    If(Box<AstNode>, Vec<AstNode>, Vec<(Box<AstNode>, Vec<AstNode>)>, Option<Vec<AstNode>>),
    Write(Box<AstNode>),
    Add(Box<AstNode>, Box<AstNode>),
    Mul(Box<AstNode>, Box<AstNode>),
    Return(Option<Box<AstNode>>),
    StringLit(String),
    NumberLit(f64),
    BoolLit(bool),
    Ident(String),
    Call(String, Vec<AstNode>),
    Binary(Box<AstNode>, String, Box<AstNode>),
    Unary(String, Box<AstNode>),
}
