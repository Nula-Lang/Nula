// src/parser.rs - Parser implementation

use crate::ast::Ast;

#[derive(Debug, Clone)]
enum Token {
    Ident(String),
    Number(f64),
    StringLit(String),
    Operator(String),
    Keyword(String),
    Symbol(String),
    Eof,
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(code: &str) -> Self {
        // Tokenization logic (expanded)
        let mut tokens = Vec::new();
        let mut chars = code.chars().peekable();
        while chars.peek().is_some() {
            let ch = *chars.peek().unwrap();
            match ch {
                ' ' | '\t' | '\n' | '\r' => { chars.next(); continue; }
                'a'..='z' | 'A'..='Z' | '_' => {
                    let mut id = String::new();
                    while let Some(&c) = chars.peek() {
                        if c.is_alphanumeric() || c == '_' {
                            id.push(c);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    if ["if", "else", "while", "for", "fn", "var", "write"].contains(&id.as_str()) {
                        tokens.push(Token::Keyword(id));
                    } else {
                        tokens.push(Token::Ident(id));
                    }
                }
                '0'..='9' | '.' => {
                    let mut num_str = String::new();
                    let mut has_dot = false;
                    while let Some(&c) = chars.peek() {
                        if c.is_digit(10) {
                            num_str.push(c);
                            chars.next();
                        } else if c == '.' && !has_dot {
                            num_str.push(c);
                            has_dot = true;
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    tokens.push(Token::Number(num_str.parse().unwrap_or(0.0)));
                }
                '"' => {
                    chars.next();
                    let mut s = String::new();
                    while let Some(&c) = chars.peek() {
                        if c == '"' {
                            chars.next();
                            break;
                        }
                        s.push(c);
                        chars.next();
                    }
                    tokens.push(Token::StringLit(s));
                }
                '+' | '-' | '*' | '/' | '^' | '=' | '<' | '>' | '!' | '&' | '|' | '[' | ']' | '(' | ')' | '{' | '}' | ':' | ';' | ',' => {
                    let op = chars.next().unwrap().to_string();
                    tokens.push(if "+-*/^=<>!&|".contains(&op) { Token::Operator(op) } else { Token::Symbol(op) });
                }
                '@' => {
                    // Single line comment
                    chars.next();
                    while let Some(&c) = chars.peek() {
                        if c == '\n' { break; }
                        chars.next();
                    }
                }
                '!' => {
                    // Multi line comment
                    chars.next();
                    let mut depth = 1;
                    while depth > 0 && chars.peek().is_some() {
                        let c = chars.next().unwrap();
                        if c == '!' { depth -= 1; }
                    }
                }
                _ => { chars.next(); } // Ignore unknown
            }
        }
        tokens.push(Token::Eof);
        Parser { tokens, pos: 0 }
    }

    pub fn parse(&mut self) -> Vec<Ast> {
        let mut stmts = Vec::new();
        while self.pos < self.tokens.len() - 1 {
            stmts.push(self.parse_stmt());
        }
        stmts
    }

    fn parse_stmt(&mut self) -> Ast {
        match &self.peek() {
            Token::Keyword(k) if k == "var" => self.parse_var_decl(),
            Token::Keyword(k) if k == "fn" => self.parse_func_def(),
            Token::Keyword(k) if k == "if" => self.parse_if(),
            Token::Keyword(k) if k == "while" => self.parse_while(),
            Token::Keyword(k) if k == "for" => self.parse_for(),
            Token::Keyword(k) if k == "write" => self.parse_write(),
            Token::Ident(_) => self.parse_assign_or_call(),
            _ => self.parse_expr(),
        }
    }

    fn parse_var_decl(&mut self) -> Ast {
        self.next(); // var
        let name = if let Token::Ident(n) = self.next() { n } else { panic!("Expected ident"); };
        if let Token::Operator(op) = self.peek() {
            if op == "=" {
                self.next();
                let value = self.parse_expr();
                Ast::VarDecl(name, Box::new(value))
            } else {
                panic!("Expected =");
            }
        } else {
            panic!("Expected =");
        }
    }

    fn parse_assign_or_call(&mut self) -> Ast {
        let name = if let Token::Ident(n) = self.next() { n } else { unreachable!() };
        match self.peek() {
            Token::Symbol(s) if s == "(" => Ast::FuncCall(name, self.parse_args()),
            Token::Operator(op) if op == "=" => {
                self.next();
                Ast::Assign(name, Box::new(self.parse_expr()))
            }
            Token::Symbol(s) if s == "[" => {
                self.next();
                let index = self.parse_expr();
                self.expect_symbol("]");
                Ast::Index(name, Box::new(index))
            }
            _ => Ast::Var(name),
        }
    }

    fn parse_func_def(&mut self) -> Ast {
        self.next(); // fn
        let name = if let Token::Ident(n) = self.next() { n } else { panic!("Expected func name"); };
        self.expect_symbol("(");
        let mut params = Vec::new();
        while !matches!(&self.peek(), Token::Symbol(s) if s == ")") {
            if let Token::Ident(p) = self.next() { params.push(p); }
            if matches!(&self.peek(), Token::Symbol(s) if s == ",") { self.next(); }
        }
        self.expect_symbol(")");
        self.expect_symbol("{");
        let body = self.parse_block();
        self.expect_symbol("}");
        Ast::FuncDef(name, params, body)
    }

    fn parse_if(&mut self) -> Ast {
        self.next(); // if
        let cond = self.parse_expr();
        self.expect_symbol("{");
        let then = self.parse_block();
        self.expect_symbol("}");
        let els = if matches!(&self.peek(), Token::Keyword(k) if k == "else") {
            self.next();
            self.expect_symbol("{");
            let e = self.parse_block();
            self.expect_symbol("}");
            Some(e)
        } else {
            None
        };
        Ast::If(Box::new(cond), then, els)
    }

    fn parse_while(&mut self) -> Ast {
        self.next(); // while
        let cond = self.parse_expr();
        self.expect_symbol("{");
        let body = self.parse_block();
        self.expect_symbol("}");
        Ast::While(Box::new(cond), body)
    }

    fn parse_for(&mut self) -> Ast {
        self.next(); // for
        let var = if let Token::Ident(v) = self.next() { v } else { panic!("Expected var"); };
        self.expect_keyword("in");
        let start = self.parse_expr();
        self.expect_operator("..");
        let end = self.parse_expr();
        self.expect_symbol("{");
        let body = self.parse_block();
        self.expect_symbol("}");
        Ast::For(var, Box::new(start), Box::new(end), body)
    }

    fn parse_write(&mut self) -> Ast {
        self.next(); // write
        Ast::FuncCall("write".to_string(), vec![self.parse_expr()])
    }

    fn parse_block(&mut self) -> Vec<Ast> {
        let mut block = Vec::new();
        while !matches!(&self.peek(), Token::Symbol(s) if s == "}") && !matches!(&self.peek(), Token::Eof) {
            block.push(self.parse_stmt());
        }
        block
    }

    fn parse_expr(&mut self) -> Ast {
        self.parse_add()
    }

    fn parse_add(&mut self) -> Ast {
        let mut left = self.parse_mul();
        while matches!(&self.peek(), Token::Operator(op) if ["+", "-"].contains(&op.as_str())) {
            let op = self.next_operator();
            let right = self.parse_mul();
            left = Ast::BinOp(op, Box::new(left), Box::new(right));
        }
        left
    }

    fn parse_mul(&mut self) -> Ast {
        let mut left = self.parse_pow();
        while matches!(&self.peek(), Token::Operator(op) if ["*", "/"].contains(&op.as_str())) {
            let op = self.next_operator();
            let right = self.parse_pow();
            left = Ast::BinOp(op, Box::new(left), Box::new(right));
        }
        left
    }

    fn parse_pow(&mut self) -> Ast {
        let mut left = self.parse_primary();
        while matches!(&self.peek(), Token::Operator(op) if op == "^") {
            self.next();
            let right = self.parse_primary();
            left = Ast::BinOp("^".to_string(), Box::new(left), Box::new(right));
        }
        left
    }

    fn parse_primary(&mut self) -> Ast {
        match self.peek().clone() {
            Token::Number(n) => { self.next(); Ast::Literal(n) }
            Token::StringLit(s) => { self.next(); Ast::StrLit(s) }
            Token::Ident(id) => self.parse_assign_or_call(),
            Token::Symbol(s) if s == "(" => {
                self.next();
                let expr = self.parse_expr();
                self.expect_symbol(")");
                expr
            }
            Token::Symbol(s) if s == "[" => self.parse_array(),
            _ => panic!("Unexpected token: {:?}", self.peek()),
        }
    }

    fn parse_array(&mut self) -> Ast {
        self.next(); // [
        let mut elements = Vec::new();
        while !matches!(&self.peek(), Token::Symbol(s) if s == "]") {
            elements.push(self.parse_expr());
            if matches!(&self.peek(), Token::Symbol(s) if s == ",") { self.next(); }
        }
        self.expect_symbol("]");
        Ast::Array(elements)
    }

    fn parse_args(&mut self) -> Vec<Ast> {
        self.expect_symbol("(");
        let mut args = Vec::new();
        while !matches!(&self.peek(), Token::Symbol(s) if s == ")") {
            args.push(self.parse_expr());
            if matches!(&self.peek(), Token::Symbol(s) if s == ",") { self.next(); }
        }
        self.expect_symbol(")");
        args
    }

    fn next(&mut self) -> Token {
        let tok = self.tokens[self.pos].clone();
        self.pos += 1;
        tok
    }

    fn peek(&self) -> Token {
        self.tokens[self.pos].clone()
    }

    fn expect_symbol(&mut self, sym: &str) {
        if matches!(&self.peek(), Token::Symbol(s) if s == sym) {
            self.next();
        } else {
            panic!("Expected symbol {}, got {:?}", sym, self.peek());
        }
    }

    fn expect_operator(&mut self, op: &str) {
        if matches!(&self.peek(), Token::Operator(o) if o == op) {
            self.next();
        } else {
            panic!("Expected operator {}, got {:?}", op, self.peek());
        }
    }

    fn expect_keyword(&mut self, kw: &str) {
        if matches!(&self.peek(), Token::Keyword(k) if k == kw) {
            self.next();
        } else {
            panic!("Expected keyword {}, got {:?}", kw, self.peek());
        }
    }

    fn next_operator(&mut self) -> String {
        if let Token::Operator(op) = self.next() { op } else { panic!("Expected operator"); }
    }
}
