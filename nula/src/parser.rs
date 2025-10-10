use crate::ast::AstNode;
use crate::lexer::Token;
use std::path::Path;

pub fn parse_nula_file(path: &Path) -> Result<AstNode, String> {
    let code = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => return Err(e.to_string()),
    };
    println!("DEBUG: Parsing code:\n{}", code);
    let tokens = crate::lexer::lex(&code);
    println!("DEBUG: Parsed tokens: {:?}", tokens);
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn current_token(&self) -> Token {
        if self.pos < self.tokens.len() {
            self.tokens[self.pos].clone()
        } else {
            Token::Eof
        }
    }

    fn advance(&mut self) {
        self.pos += 1;
    }

    pub fn match_keyword(&mut self, kw: &str) -> bool {
        match self.current_token() {
            Token::Keyword(ref s) if s == kw => {
                self.advance();
                true
            }
            _ => false
        }
    }

    pub fn match_punctuation(&mut self, p: char) -> bool {
        match self.current_token() {
            Token::Punctuation(c) if c == p => {
                self.advance();
                true
            }
            _ => false
        }
    }

    pub fn match_op(&mut self, ops: &[&str]) -> Option<String> {
        match self.current_token() {
            Token::Operator(ref s) if ops.contains(&s.as_str()) => {
                let op = s.clone();
                self.advance();
                Some(op)
            }
            Token::Keyword(ref s) if ops.contains(&s.as_str()) => {
                let op = s.clone();
                self.advance();
                Some(op)
            }
            _ => None,
        }
    }

    pub fn consume_ident(&mut self) -> String {
        match self.current_token() {
            Token::Ident(s) => {
                self.advance();
                s
            }
            _ => String::new()
        }
    }

    pub fn consume_string(&mut self) -> String {
        match self.current_token() {
            Token::StringLit(s) => {
                self.advance();
                s
            }
            _ => String::new()
        }
    }

    pub fn parse_program(&mut self) -> Result<AstNode, String> {
        let mut nodes = vec![];
        while !matches!(self.current_token(), Token::Eof) {
            nodes.push(self.parse_statement()?);
        }
        Ok(AstNode::Program(nodes))
    }

    fn parse_statement(&mut self) -> Result<AstNode, String> {
        if self.match_keyword("let") {
            self.parse_variable_decl()
        } else if self.match_keyword("set") {
            self.parse_assignment()
        } else if self.match_keyword("fn") {
            self.parse_function_def()
        } else if self.match_keyword("for") {
            self.parse_for_loop()
        } else if self.match_keyword("while") {
            self.parse_while_loop()
        } else if self.match_keyword("if") {
            self.parse_conditional()
        } else if self.match_keyword("write") {
            self.parse_write_stmt()
        } else if self.match_keyword("add") {
            self.parse_add_stmt()
        } else if self.match_keyword("mul") {
            self.parse_mul_stmt()
        } else if self.match_keyword("return") {
            self.parse_return_stmt()
        } else if self.match_keyword("import") {
            self.parse_import_stmt()
        } else if self.match_punctuation('<') {
            self.parse_dependency()
        } else if self.match_punctuation('#') {
            self.parse_translation()
        } else {
            self.parse_expression()
        }
    }

    fn parse_variable_decl(&mut self) -> Result<AstNode, String> {
        let ident = self.consume_ident();
        let expr = self.parse_expression()?;
        Ok(AstNode::VariableDecl(ident, Box::new(expr)))
    }

    fn parse_assignment(&mut self) -> Result<AstNode, String> {
        let ident = self.consume_ident();
        let expr = self.parse_expression()?;
        Ok(AstNode::Assignment(ident, Box::new(expr)))
    }

    fn parse_function_def(&mut self) -> Result<AstNode, String> {
        let name = self.consume_ident();
        self.match_punctuation('(');
        let mut params = vec![];
        while !self.match_punctuation(')') {
            params.push(self.consume_ident());
            self.match_punctuation(',');
        }
        self.match_punctuation('{');
        let mut body = vec![];
        while !self.match_punctuation('}') {
            body.push(self.parse_statement()?);
        }
        Ok(AstNode::FunctionDef(name, params, body))
    }

    fn parse_for_loop(&mut self) -> Result<AstNode, String> {
        let var = self.consume_ident();
        self.match_keyword("in");
        let iter = self.parse_expression()?;
        self.match_punctuation('{');
        let mut body = vec![];
        while !self.match_punctuation('}') {
            body.push(self.parse_statement()?);
        }
        Ok(AstNode::ForLoop(var, Box::new(iter), body))
    }

    fn parse_while_loop(&mut self) -> Result<AstNode, String> {
        let cond = self.parse_expression()?;
        self.match_punctuation('{');
        let mut body = vec![];
        while !self.match_punctuation('}') {
            body.push(self.parse_statement()?);
        }
        Ok(AstNode::WhileLoop(Box::new(cond), body))
    }

    fn parse_conditional(&mut self) -> Result<AstNode, String> {
        let cond = self.parse_expression()?;
        self.match_punctuation('{');
        let mut body = vec![];
        while !self.match_punctuation('}') {
            body.push(self.parse_statement()?);
        }
        let mut else_ifs = vec![];
        let mut else_body = None;

        while self.match_keyword("else") {
            if self.match_keyword("if") {
                let ei_cond = self.parse_expression()?;
                self.match_punctuation('{');
                let mut ei_body = vec![];
                while !self.match_punctuation('}') {
                    ei_body.push(self.parse_statement()?);
                }
                else_ifs.push((Box::new(ei_cond), ei_body));
            } else {
                self.match_punctuation('{');
                let mut eb = vec![];
                while !self.match_punctuation('}') {
                    eb.push(self.parse_statement()?);
                }
                else_body = Some(eb);
                break;
            }
        }
        Ok(AstNode::If(Box::new(cond), body, else_ifs, else_body))
    }

    fn parse_write_stmt(&mut self) -> Result<AstNode, String> {
        let expr = self.parse_expression()?;
        Ok(AstNode::Write(Box::new(expr)))
    }

    fn parse_add_stmt(&mut self) -> Result<AstNode, String> {
        let left = self.parse_expression()?;
        let right = self.parse_expression()?;
        Ok(AstNode::Add(Box::new(left), Box::new(right)))
    }

    fn parse_mul_stmt(&mut self) -> Result<AstNode, String> {
        let left = self.parse_expression()?;
        let right = self.parse_expression()?;
        Ok(AstNode::Mul(Box::new(left), Box::new(right)))
    }

    fn parse_return_stmt(&mut self) -> Result<AstNode, String> {
        let expr = if !matches!(self.current_token(), Token::Eof) {
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };
        Ok(AstNode::Return(expr))
    }

    fn parse_import_stmt(&mut self) -> Result<AstNode, String> {
        let ident = self.consume_ident();
        Ok(AstNode::Import(ident))
    }

    fn parse_dependency(&mut self) -> Result<AstNode, String> {
        let dep = self.consume_ident();
        self.match_punctuation('>');
        Ok(AstNode::Dependency(dep))
    }

    fn parse_translation(&mut self) -> Result<AstNode, String> {
        if self.match_op(&["="]).is_none() {
            return Err("Expected '=' after '#'".to_string());
        }
        let ident = self.consume_ident();
        let code_block = self.consume_string();
        Ok(AstNode::Translation(ident, code_block))
    }

    fn parse_expression(&mut self) -> Result<AstNode, String> {
        self.parse_logic_or()
    }

    fn parse_logic_or(&mut self) -> Result<AstNode, String> {
        let mut left = self.parse_logic_and()?;
        while let Some(op) = self.match_op(&["or", "||"]) {
            let right = self.parse_logic_and()?;
            left = AstNode::Binary(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    fn parse_logic_and(&mut self) -> Result<AstNode, String> {
        let mut left = self.parse_equality()?;
        while let Some(op) = self.match_op(&["and", "&&"]) {
            let right = self.parse_equality()?;
            left = AstNode::Binary(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<AstNode, String> {
        let mut left = self.parse_comparison()?;
        while let Some(op) = self.match_op(&["==", "!="]) {
            let right = self.parse_comparison()?;
            left = AstNode::Binary(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<AstNode, String> {
        let mut left = self.parse_addition()?;
        while let Some(op) = self.match_op(&["<", ">", "<=", ">="]) {
            let right = self.parse_addition()?;
            left = AstNode::Binary(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    fn parse_addition(&mut self) -> Result<AstNode, String> {
        let mut left = self.parse_multiplication()?;
        while let Some(op) = self.match_op(&["+", "-"]) {
            let right = self.parse_multiplication()?;
            left = AstNode::Binary(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    fn parse_multiplication(&mut self) -> Result<AstNode, String> {
        let mut left = self.parse_unary()?;
        while let Some(op) = self.match_op(&["*", "/"]) {
            let right = self.parse_unary()?;
            left = AstNode::Binary(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<AstNode, String> {
        if let Some(op) = self.match_op(&["-", "not", "!"]) {
            let expr = self.parse_unary()?;
            Ok(AstNode::Unary(op, Box::new(expr)))
        } else {
            self.parse_primary()
        }
    }

    fn parse_primary(&mut self) -> Result<AstNode, String> {
        let token = self.current_token();
        match token {
            Token::StringLit(s) => {
                self.advance();
                Ok(AstNode::StringLit(s))
            }
            Token::NumberLit(n) => {
                self.advance();
                Ok(AstNode::NumberLit(n))
            }
            Token::BoolLit(b) => {
                self.advance();
                Ok(AstNode::BoolLit(b))
            }
            Token::Ident(s) => {
                self.advance();
                if self.match_punctuation('(') {
                    let mut args = vec![];
                    while !self.match_punctuation(')') {
                        args.push(self.parse_expression()?);
                        self.match_punctuation(',');
                    }
                    Ok(AstNode::Call(s, args))
                } else {
                    Ok(AstNode::Ident(s))
                }
            }
            Token::Punctuation('(') => {
                self.advance();
                let expr = self.parse_expression()?;
                if !self.match_punctuation(')') {
                    return Err("Expected ')'".to_string());
                }
                Ok(expr)
            }
            _ => Err(format!("Unexpected token in primary: {:?}", token)),
        }
    }
}
