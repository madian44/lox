use crate::{expr, location, reporter, token};
use std::collections::linked_list::IntoIter;
use std::collections::LinkedList;
use std::iter::Peekable;

type TokenIterator = Peekable<IntoIter<token::Token>>;
pub struct ParseError {
    message: String,
}

pub fn parse(
    reporter: &dyn reporter::Reporter,
    tokens: LinkedList<token::Token>,
) -> Option<expr::Expr> {
    let mut parser = Parser::new();

    let result = parser.parse(reporter, tokens);
    if let Err(e) = &result {
        reporter.add_message(&format!("[parser] {}", e.message));
        None
    } else {
        result.ok()
    }
}

struct Data {
    equality_tokens: Vec<token::TokenType>,
    comparison_tokens: Vec<token::TokenType>,
    factor_tokens: Vec<token::TokenType>,
    term_tokens: Vec<token::TokenType>,
    unary_tokens: Vec<token::TokenType>,
    primary_tokens: Vec<token::TokenType>,
    start_of_group_tokens: Vec<token::TokenType>,
}

impl Data {
    fn new() -> Self {
        let equality_tokens = vec![token::TokenType::BangEqual, token::TokenType::EqualEqual];
        let comparison_tokens = vec![
            token::TokenType::Greater,
            token::TokenType::GreaterEqual,
            token::TokenType::Less,
            token::TokenType::LessEqual,
        ];
        let factor_tokens = vec![token::TokenType::Slash, token::TokenType::Star];
        let term_tokens = vec![token::TokenType::Minus, token::TokenType::Plus];
        let unary_tokens = vec![token::TokenType::Bang, token::TokenType::Minus];
        let primary_tokens = vec![
            token::TokenType::False,
            token::TokenType::True,
            token::TokenType::Nil,
            token::TokenType::Number,
            token::TokenType::String,
        ];
        let start_of_group_tokens = vec![token::TokenType::LeftParen];
        Data {
            equality_tokens,
            comparison_tokens,
            factor_tokens,
            term_tokens,
            unary_tokens,
            primary_tokens,
            start_of_group_tokens,
        }
    }
}

struct Parser {
    current_token: Option<token::Token>,
    last_location: Option<location::FileLocation>,
}

impl Parser {
    fn new() -> Self {
        Parser {
            current_token: None,
            last_location: None,
        }
    }

    fn take_current_token(&mut self) -> Result<token::Token, ParseError> {
        match self.current_token.take() {
            Some(token) => Ok(token),
            _ => Err(ParseError {
                message: "no current token".to_string(),
            }),
        }
    }

    fn parse(
        &mut self,
        reporter: &dyn reporter::Reporter,
        tokens: LinkedList<token::Token>,
    ) -> Result<expr::Expr, ParseError> {
        let mut tokens = tokens.into_iter().peekable();
        let data = Data::new();
        if self.is_at_end(&mut tokens) {
            return Err(ParseError {
                message: "Nothing to parse".to_string(),
            });
        }
        self.expression(reporter, &mut tokens, &data)
    }

    fn expression(
        &mut self,
        reporter: &dyn reporter::Reporter,
        tokens: &mut TokenIterator,
        data: &Data,
    ) -> Result<expr::Expr, ParseError> {
        self.equality(reporter, tokens, data)
    }

    fn equality(
        &mut self,
        reporter: &dyn reporter::Reporter,
        tokens: &mut TokenIterator,
        data: &Data,
    ) -> Result<expr::Expr, ParseError> {
        let mut expr = self.comparison(reporter, tokens, data)?;

        while self.match_next_token(tokens, &data.equality_tokens) {
            let operator = self.take_current_token()?;
            let right = self.comparison(reporter, tokens, data)?;
            expr = expr::Expr::build_binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn comparison(
        &mut self,
        reporter: &dyn reporter::Reporter,
        tokens: &mut TokenIterator,
        data: &Data,
    ) -> Result<expr::Expr, ParseError> {
        let mut expr = self.term(reporter, tokens, data)?;

        while self.match_next_token(tokens, &data.comparison_tokens) {
            let operator = self.take_current_token()?;
            let right = self.term(reporter, tokens, data)?;
            expr = expr::Expr::build_binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn term(
        &mut self,
        reporter: &dyn reporter::Reporter,
        tokens: &mut TokenIterator,
        data: &Data,
    ) -> Result<expr::Expr, ParseError> {
        let mut expr = self.factor(reporter, tokens, data)?;

        while self.match_next_token(tokens, &data.term_tokens) {
            let operator = self.take_current_token()?;
            let right = self.factor(reporter, tokens, data)?;
            expr = expr::Expr::build_binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn factor(
        &mut self,
        reporter: &dyn reporter::Reporter,
        tokens: &mut TokenIterator,
        data: &Data,
    ) -> Result<expr::Expr, ParseError> {
        let mut expr = self.unary(reporter, tokens, data)?;

        while self.match_next_token(tokens, &data.factor_tokens) {
            let operator = self.take_current_token()?;
            let right = self.unary(reporter, tokens, data)?;
            expr = expr::Expr::build_binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn unary(
        &mut self,
        reporter: &dyn reporter::Reporter,
        tokens: &mut TokenIterator,
        data: &Data,
    ) -> Result<expr::Expr, ParseError> {
        if self.match_next_token(tokens, &data.unary_tokens) {
            let operator = self.take_current_token()?;
            let right = self.unary(reporter, tokens, data)?;
            return Ok(expr::Expr::build_unary(operator, right));
        }
        self.primary(reporter, tokens, data)
    }

    fn primary(
        &mut self,
        reporter: &dyn reporter::Reporter,
        tokens: &mut TokenIterator,
        data: &Data,
    ) -> Result<expr::Expr, ParseError> {
        if self.match_next_token(tokens, &data.primary_tokens) {
            return Ok(expr::Expr::build_literal(self.take_current_token()?));
        }

        if self.match_next_token(tokens, &data.start_of_group_tokens) {
            let expr = self.expression(reporter, tokens, data)?;
            self.consume(
                reporter,
                tokens,
                &token::TokenType::RightParen,
                "expect ')' after expression",
            )?;
            return Ok(expr::Expr::build_grouping(expr));
        }

        self.add_diagnostic(reporter, tokens, "Primary expression expected")
    }

    fn consume(
        &mut self,
        reporter: &dyn reporter::Reporter,
        tokens: &mut TokenIterator,
        token_to_consume: &token::TokenType,
        message: &str,
    ) -> Result<(), ParseError> {
        if self.check_next_token(tokens, token_to_consume) {
            self.advance(tokens);
            return Ok(());
        }

        self.add_diagnostic(reporter, tokens, message)?;
        Ok(())
    }

    fn match_next_token(
        &mut self,
        tokens: &mut TokenIterator,
        token_types: &[token::TokenType],
    ) -> bool {
        if token_types.iter().any(|t| self.check_next_token(tokens, t)) {
            self.advance(tokens);
            true
        } else {
            false
        }
    }

    fn check_next_token(
        &self,
        tokens: &mut TokenIterator,
        type_to_check: &token::TokenType,
    ) -> bool {
        if self.is_at_end(tokens) {
            false
        } else {
            match tokens.peek() {
                Some(t) => t.token_type == *type_to_check,
                None => false,
            }
        }
    }

    fn is_at_end(&self, tokens: &mut TokenIterator) -> bool {
        match tokens.peek() {
            Some(t) => t.token_type == token::TokenType::Eof,
            None => true,
        }
    }

    fn advance(&mut self, tokens: &mut TokenIterator) {
        self.current_token = tokens.next();
        if self.current_token.is_some() {
            self.last_location = Some(self.current_token.as_ref().unwrap().start);
        }
    }

    fn add_diagnostic(
        &mut self,
        reporter: &dyn reporter::Reporter,
        tokens: &mut TokenIterator,
        message: &str,
    ) -> Result<expr::Expr, ParseError> {
        let location = self
            .get_nearby_location(tokens)
            .unwrap_or(location::FileLocation::new(0, 0));
        reporter.add_diagnostic(&location, &location, message);
        Err(ParseError {
            message: message.to_string(),
        })
    }

    fn get_nearby_location(
        &mut self,
        tokens: &mut TokenIterator,
    ) -> Option<location::FileLocation> {
        if let Some(location) = &self.last_location {
            return Some(*location);
        }
        if let Some(token) = tokens.peek() {
            return Some(token.start);
        }
        None
    }

    fn synchronize(&mut self, tokens: &mut TokenIterator) {
        // untested
        while !self.is_at_end(tokens) {
            if let Some(token) = tokens.peek() {
                if token.token_type == token::TokenType::Semicolon {
                    return;
                }
            }

            match self.current_token.as_ref().unwrap().token_type {
                token::TokenType::Class
                | token::TokenType::Fun
                | token::TokenType::Var
                | token::TokenType::For
                | token::TokenType::If
                | token::TokenType::While
                | token::TokenType::Print
                | token::TokenType::Return => return,
                _ => (),
            }

            self.advance(tokens);
        }
    }
}
#[cfg(test)]
mod test {
    use super::*;
    use crate::reporter::test::TestReporter;
    use crate::FileLocation;
    use crate::{ast_printer, Reporter};

    #[test]
    fn production_tests() {
        let reporter = TestReporter::build();

        let blank_location = FileLocation::new(0, 0);

        let tests = vec![
            (
                "(a string)",
                vec![token::Token::new(
                    token::TokenType::String,
                    "\"a string\"",
                    blank_location,
                    blank_location,
                    token::Literal::String("a string".to_string()),
                )],
            ),
            (
                "(+ (a string) (10))",
                vec![
                    token::Token::new(
                        token::TokenType::String,
                        "\"a string\"",
                        blank_location,
                        blank_location,
                        token::Literal::String("a string".to_string()),
                    ),
                    token::Token::new(
                        token::TokenType::Plus,
                        "+",
                        blank_location,
                        blank_location,
                        token::Literal::None,
                    ),
                    token::Token::new(
                        token::TokenType::Number,
                        "10",
                        blank_location,
                        blank_location,
                        token::Literal::Number(10f64),
                    ),
                ],
            ),
            (
                "(group (+ (a string) (10)))",
                vec![
                    token::Token::new(
                        token::TokenType::LeftParen,
                        "(",
                        blank_location,
                        blank_location,
                        token::Literal::None,
                    ),
                    token::Token::new(
                        token::TokenType::String,
                        "\"a string\"",
                        blank_location,
                        blank_location,
                        token::Literal::String("a string".to_string()),
                    ),
                    token::Token::new(
                        token::TokenType::Plus,
                        "+",
                        blank_location,
                        blank_location,
                        token::Literal::None,
                    ),
                    token::Token::new(
                        token::TokenType::Number,
                        "10",
                        blank_location,
                        blank_location,
                        token::Literal::Number(10f64),
                    ),
                    token::Token::new(
                        token::TokenType::RightParen,
                        ")",
                        blank_location,
                        blank_location,
                        token::Literal::None,
                    ),
                ],
            ),
            (
                "(== (10) (11))",
                vec![
                    token::Token::new(
                        token::TokenType::Number,
                        "10",
                        blank_location,
                        blank_location,
                        token::Literal::Number(10f64),
                    ),
                    token::Token::new(
                        token::TokenType::EqualEqual,
                        "==",
                        blank_location,
                        blank_location,
                        token::Literal::None,
                    ),
                    token::Token::new(
                        token::TokenType::Number,
                        "11",
                        blank_location,
                        blank_location,
                        token::Literal::Number(11f64),
                    ),
                ],
            ),
            (
                "(> (10) (11))",
                vec![
                    token::Token::new(
                        token::TokenType::Number,
                        "10",
                        blank_location,
                        blank_location,
                        token::Literal::Number(10f64),
                    ),
                    token::Token::new(
                        token::TokenType::Greater,
                        ">",
                        blank_location,
                        blank_location,
                        token::Literal::None,
                    ),
                    token::Token::new(
                        token::TokenType::Number,
                        "11",
                        blank_location,
                        blank_location,
                        token::Literal::Number(11f64),
                    ),
                ],
            ),
            (
                "(* (10) (11))",
                vec![
                    token::Token::new(
                        token::TokenType::Number,
                        "10",
                        blank_location,
                        blank_location,
                        token::Literal::Number(10f64),
                    ),
                    token::Token::new(
                        token::TokenType::Star,
                        "*",
                        blank_location,
                        blank_location,
                        token::Literal::None,
                    ),
                    token::Token::new(
                        token::TokenType::Number,
                        "11",
                        blank_location,
                        blank_location,
                        token::Literal::Number(11f64),
                    ),
                ],
            ),
            (
                "(! (! (10)))",
                vec![
                    token::Token::new(
                        token::TokenType::Bang,
                        "!",
                        blank_location,
                        blank_location,
                        token::Literal::None,
                    ),
                    token::Token::new(
                        token::TokenType::Bang,
                        "!",
                        blank_location,
                        blank_location,
                        token::Literal::None,
                    ),
                    token::Token::new(
                        token::TokenType::Number,
                        "10",
                        blank_location,
                        blank_location,
                        token::Literal::Number(10f64),
                    ),
                ],
            ),
        ];

        for (expected_parse, tokens) in tests {
            reporter.reset();

            let tokens: LinkedList<token::Token> = tokens.into_iter().collect();
            match parse(&reporter, tokens) {
                Some(expr) => assert_eq!(ast_printer::print(&expr), expected_parse),
                _ => {
                    reporter.print_contents();
                    panic!("unexpected failure")
                }
            };
            assert!(!reporter.has_diagnostics());
        }
    }

    #[test]
    fn errors() {
        let reporter = TestReporter::build();

        let blank_location = FileLocation::new(0, 0);

        let tests = vec![
            (
                "Primary expression expected",
                vec![
                    token::Token::new(
                        token::TokenType::Slash,
                        "/",
                        blank_location,
                        blank_location,
                        token::Literal::None,
                    ),
                    token::Token::new(
                        token::TokenType::Number,
                        "\"10\"",
                        blank_location,
                        blank_location,
                        token::Literal::Number(10f64),
                    ),
                ],
            ),
            (
                "expect ')' after expression",
                vec![
                    token::Token::new(
                        token::TokenType::LeftParen,
                        "(",
                        blank_location,
                        blank_location,
                        token::Literal::None,
                    ),
                    token::Token::new(
                        token::TokenType::Number,
                        "\"10\"",
                        blank_location,
                        blank_location,
                        token::Literal::Number(10f64),
                    ),
                ],
            ),
        ];

        for (expected_message, tokens) in tests {
            reporter.reset();
            let tokens: LinkedList<token::Token> = tokens.into_iter().collect();
            if let Some(expr) = parse(&reporter, tokens) {
                panic!("Unexpected expression found: {}", ast_printer::print(&expr));
            };
            assert!(reporter.has_diagnostics());
            assert_eq!(reporter.diagnostics_len(), 1);

            assert_eq!(
                reporter
                    .diagnostic_get(0)
                    .expect("missing diagnostic")
                    .message,
                expected_message
            );
        }
    }
}
