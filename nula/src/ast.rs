use indextree::{Arena, NodeId};

#[derive(Debug, Clone)]
pub enum Type {
    Int,
    Str,
    Array(Box<Type>),
    Unknown,
}

#[derive(Debug, Clone)]
pub enum AstNode {
    Write(Expr),
    Comment(String),
    Dependency(String),
    ForeignBlock { lang: String, code: String },
    Let { name: String, value: Expr, ty: Type },
    For { var: String, range: (Expr, Expr), body: Vec<AstNode> },
    If { cond: Expr, body: Vec<AstNode> },
    While { cond: Expr, body: Vec<AstNode> },
    Fn { name: String, params: Vec<(String, Type)>, body: Vec<AstNode>, ret: Box<Expr>, ret_ty: Type },
    Expr(Expr),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Int(i32),
    Str(String),
    Var(String),
    BinOp { op: char, left: Box<Expr>, right: Box<Expr> },
    Call { name: String, args: Vec<Expr> },
    Array(Vec<Expr>),
    Index { arr: Box<Expr>, idx: Box<Expr> },
    Length(Box<Expr>),
}

pub struct AstTree {
    pub arena: Arena<AstNode>,
    pub root: NodeId,
}

impl AstTree {
    pub fn new() -> Self {
        let mut arena = Arena::new();
        let root = arena.new_node(AstNode::Expr(Expr::Int(0))); // Dummy root
        AstTree { arena, root }
    }

    pub fn infer_types(&mut self) {
        // Proste type inference: traverse and set types
        // Dla przykładu, assume Int dla liczb, etc.
    }
}
