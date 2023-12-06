use crate::location;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TokenType {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Semicolon,
    Slash,
    Star,

    // One or two character tokens.
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    // Literals.
    Identifier,
    String,
    Number,

    // Keywords.
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,

    Eof,
}

pub struct Keywords<'a> {
    keywords: HashMap<&'a str, TokenType>,
}

impl<'a> Keywords<'a> {
    pub fn build() -> Self {
        let mut keywords = HashMap::new();
        keywords.insert("and", TokenType::And);
        keywords.insert("class", TokenType::Class);
        keywords.insert("else", TokenType::Else);
        keywords.insert("false", TokenType::False);
        keywords.insert("for", TokenType::For);
        keywords.insert("fun", TokenType::Fun);
        keywords.insert("if", TokenType::If);
        keywords.insert("nil", TokenType::Nil);
        keywords.insert("or", TokenType::Or);
        keywords.insert("print", TokenType::Print);
        keywords.insert("return", TokenType::Return);
        keywords.insert("super", TokenType::Super);
        keywords.insert("this", TokenType::This);
        keywords.insert("true", TokenType::True);
        keywords.insert("var", TokenType::Var);
        keywords.insert("while", TokenType::While);

        Keywords { keywords }
    }

    pub fn get_keyword(&self, lexeme: &str) -> Option<TokenType> {
        self.keywords.get(lexeme).copied()
    }
}

#[derive(Debug, PartialEq)]
pub enum Literal {
    String(String),
    Number(f64),
    True,
    False,
    Nil,
}

pub fn get_keyword_literal(token_type: &TokenType) -> Option<Literal> {
    match token_type {
        TokenType::False => Some(Literal::False),
        TokenType::True => Some(Literal::True),
        TokenType::Nil => Some(Literal::Nil),
        _ => None,
    }
}

#[derive(Debug, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Option<Literal>,
    pub start: location::FileLocation,
    pub end: location::FileLocation,
}

impl Token {
    pub fn new(
        token_type: TokenType,
        lexeme: &str,
        start: location::FileLocation,
        end: location::FileLocation,
        literal: Option<Literal>,
    ) -> Self {
        Token {
            token_type,
            lexeme: lexeme.to_string(),
            start,
            end,
            literal,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} '{}'", self.token_type, self.lexeme)
    }
}
