#[derive(Debug, Clone)]
pub enum AstNode {
    Write(String),
    Comment(String),
    Dependency(String),
    ForeignBlock { lang: String, code: String },
    Let { name: String, value: Expr },
    For { var: String, range: (i32, i32), body: Vec<AstNode> },
    If { cond: Expr, body: Vec<AstNode> },
    Fn { name: String, params: Vec<String>, body: Vec<AstNode>, ret: Box<Expr> },
    Expr(Expr),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Int(i32),
    Str(String),
    Var(String),
    BinOp { op: char, left: Box<Expr>, right: Box<Expr> },
    Call { name: String, args: Vec<Expr> },
}
