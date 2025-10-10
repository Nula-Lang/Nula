#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Ident(String),
    Keyword(String),
    StringLit(String),
    NumberLit(f64),
    BoolLit(bool),
    Operator(String),
    Punctuation(char),
    Eof,
}

const KEYWORDS: &[&str] = &[
    "let", "set", "fn", "for", "in", "while", "if", "else", "write", "add", "mul", "return",
"import", "and", "or", "not",
];

struct Lexer<'a> {
    chars: std::iter::Peekable<std::str::Chars<'a>>,
    current_char: Option<char>,
}

impl<'a> Lexer<'a> {
    fn new(code: &'a str) -> Self {
        let mut chars = code.chars().peekable();
        let current_char = chars.next();
        Self { chars, current_char }
    }

    fn advance(&mut self) {
        self.current_char = self.chars.next();
    }

    fn peek(&mut self) -> Option<char> {
        self.chars.peek().cloned()
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current_char {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn lex_number(&mut self) -> Token {
        let mut num_str = String::new();
        let mut has_dot = false;
        while let Some(c) = self.current_char {
            if c.is_digit(10) {
                num_str.push(c);
                self.advance();
            } else if c == '.' && !has_dot {
                has_dot = true;
                num_str.push(c);
                self.advance();
            } else {
                break;
            }
        }
        Token::NumberLit(num_str.parse::<f64>().unwrap_or(0.0))
    }

    fn lex_string(&mut self, delimiter: char) -> Token {
        let mut str_val = String::new();
        self.advance(); // Skip opening delimiter
        while let Some(c) = self.current_char {
            if c == delimiter {
                self.advance();
                break;
            }
            str_val.push(c);
            self.advance();
        }
        Token::StringLit(str_val)
    }

    fn lex_ident_or_keyword(&mut self) -> Token {
        let mut id_str = String::new();
        while let Some(c) = self.current_char {
            if c.is_alphanumeric() || c == '_' {
                id_str.push(c);
                self.advance();
            } else {
                break;
            }
        }
        match id_str.as_str() {
            "true" => Token::BoolLit(true),
            "false" => Token::BoolLit(false),
            kw if KEYWORDS.contains(&kw) => Token::Keyword(id_str),
            _ => Token::Ident(id_str),
        }
    }

    fn lex_operator_or_punctuation(&mut self) -> Token {
        let mut op_str = String::new();
        if let Some(c) = self.current_char {
            op_str.push(c);
            self.advance();
            if let Some(next_c) = self.peek() {
                let potential_two_char = format!("{}{}", c, next_c);
                if ["==", "!=", "<=", ">=", "&&", "||"].contains(&potential_two_char.as_str()) {
                    op_str.push(next_c);
                    self.advance();
                }
            }
        }
        if op_str.len() == 1 {
            if let Some(single_char) = op_str.chars().next() {
                if "(){}[],;:#".contains(single_char) {
                    return Token::Punctuation(single_char);
                }
            }
        }
        Token::Operator(op_str)
    }

    fn skip_line_comment(&mut self) {
        while let Some(c) = self.current_char {
            if c == '\n' {
                self.advance();
                break;
            }
            self.advance();
        }
    }

    fn skip_block_comment(&mut self) {
        self.advance(); // Skip *
        while let Some(c) = self.current_char {
            if c == '*' && self.peek() == Some('/') {
                self.advance();
                self.advance();
                break;
            }
            self.advance();
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        match self.current_char {
            None => Token::Eof,
            Some(c) if c.is_digit(10) => self.lex_number(),
            Some(c) if c.is_alphabetic() || c == '_' => self.lex_ident_or_keyword(),
            Some(d @ '"') | Some(d @ '\'') => self.lex_string(d),
            Some('/') => {
                self.advance();
                match self.current_char {
                    Some('/') => {
                        self.skip_line_comment();
                        self.next_token()
                    }
                    Some('*') => {
                        self.skip_block_comment();
                        self.next_token()
                    }
                    _ => Token::Operator("/".to_string()),
                }
            }
            Some(_) => self.lex_operator_or_punctuation(),
        }
    }
}

pub fn lex(code: &str) -> Vec<Token> {
    let mut lexer = Lexer::new(code);
    let mut tokens = Vec::new();
    loop {
        let token = lexer.next_token();
        if matches!(&token, Token::Eof) {
            break;
        }
        tokens.push(token);
    }
    tokens
}
