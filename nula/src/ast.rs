use indextree::{Arena, NodeId};
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum AstError {
    #[error("Type mismatch: expected {0}, found {1}")]
    TypeMismatch(String, String),
    #[error("Undefined variable or type: {0}")]
    Undefined(String),
    #[error("Generic parameter mismatch")]
    GenericMismatch,
    #[error("Trait not implemented for type {0}")]
    TraitNotImplemented(String),
    // Więcej poważnych błędów dla rozbudowanego type system
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Str,
    Bool,
    Float,
    Array(Box<Type>),
    Struct { name: String, fields: Vec<(String, Type)>, generics: Vec<Type> },
    Enum { name: String, variants: Vec<(String, Option<Type>)>, generics: Vec<Type> },
    Trait { name: String, methods: Vec<(String, FnType)> },
    Generic(String),
    Unknown,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FnType {
    params: Vec<Type>,
    ret: Box<Type>,
}

#[derive(Debug, Clone)]
pub enum AstNode {
    Write(Expr),
    Comment(String),
    Dependency(String),
    ForeignBlock { lang: String, code: String },
    Let { name: String, ty: Option<Type>, value: Expr },
    For { var: String, ty: Option<Type>, range: (Expr, Expr), body: Vec<AstNode> },
    If { cond: Expr, body: Vec<AstNode>, else_body: Option<Vec<AstNode>> },
    While { cond: Expr, body: Vec<AstNode> },
    Fn { name: String, generics: Vec<String>, params: Vec<(String, Type)>, ret_ty: Type, body: Vec<AstNode> },
    StructDef { name: String, generics: Vec<String>, fields: Vec<(String, Type)>, impl_traits: Vec<String> },
    EnumDef { name: String, generics: Vec<String>, variants: Vec<(String, Option<Type>)> },
    TraitDef { name: String, methods: Vec<(String, FnType)> },
    Impl { ty: Type, trait_name: Option<String>, methods: Vec<AstNode> },  // AstNode::Fn inside
    ClassDef { name: String, generics: Vec<String>, fields: Vec<(String, Type)>, methods: Vec<AstNode> },
    Expr(Expr),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Int(i64),  // Rozbudowano do i64
    Float(f64),
    Str(String),
    Bool(bool),
    Var { name: String, generics: Vec<Type> },
    BinOp { op: String, left: Box<Expr>, right: Box<Expr> },
    UnaryOp { op: String, expr: Box<Expr> },
    Call { callee: Box<Expr>, args: Vec<Expr> },
    Array(Vec<Expr>),
    Index { arr: Box<Expr>, idx: Box<Expr> },
    Length(Box<Expr>),
    StructLit { ty: Type, fields: Vec<(String, Expr)> },
    EnumLit { ty: Type, variant: String, value: Option<Box<Expr>> },
    FieldAccess { obj: Box<Expr>, field: String },
    MethodCall { obj: Box<Expr>, method: String, args: Vec<Expr> },
}

pub struct AstTree {
    pub arena: Arena<AstNode>,
    pub root: NodeId,
    pub type_env: std::collections::HashMap<String, Type>,
    pub trait_impls: std::collections::HashMap<(String, String), bool>,  // (type, trait) -> implemented
}

impl AstTree {
    pub fn new() -> Self {
        AstTree {
            arena: Arena::new(),
            root: Arena::new().new_node(AstNode::Expr(Expr::Int(0))),
            type_env: Default::default(),
            trait_impls: Default::default(),
        }
    }

    pub fn add_node(&mut self, parent: NodeId, node: AstNode) -> NodeId {
        let id = self.arena.new_node(node);
        parent.append(id, &mut self.arena);
        id
    }

    // Więcej metod dla manipulacji drzewem, np. resolve_generics, etc.
    pub fn resolve_generics(&mut self, generics: &[String], args: &[Type]) -> Result<(), AstError> {
        if generics.len() != args.len() {
            return Err(AstError::GenericMismatch);
        }
        // Bind generics in env
        Ok(())
    }
}
