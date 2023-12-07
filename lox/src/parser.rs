use crate::{expr, location, reporter, stmt, token};
use std::collections::linked_list::IntoIter;
use std::collections::LinkedList;
use std::iter::Peekable;

pub struct ParseError {
    message: String,
}

pub fn parse(
    reporter: &dyn reporter::Reporter,
    tokens: LinkedList<token::Token>,
) -> LinkedList<stmt::Stmt> {
    let mut parser = Parser::new(reporter, tokens);

    parser.parse()
}

struct Data {
    equality_tokens: Vec<token::TokenType>,
    comparison_tokens: Vec<token::TokenType>,
    factor_tokens: Vec<token::TokenType>,
    term_tokens: Vec<token::TokenType>,
    unary_tokens: Vec<token::TokenType>,
    primary_tokens: Vec<token::TokenType>,
    start_of_group_tokens: Vec<token::TokenType>,
    print_tokens: Vec<token::TokenType>,
    declaration_tokens: Vec<token::TokenType>,
    assignment_tokens: Vec<token::TokenType>,
    identifier_tokens: Vec<token::TokenType>,
    block_tokens: Vec<token::TokenType>,
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
        let print_tokens = vec![token::TokenType::Print];
        let declaration_tokens = vec![token::TokenType::Var];
        let assignment_tokens = vec![token::TokenType::Equal];
        let identifier_tokens = vec![token::TokenType::Identifier];
        let block_tokens = vec![token::TokenType::LeftBrace];
        Data {
            equality_tokens,
            comparison_tokens,
            factor_tokens,
            term_tokens,
            unary_tokens,
            primary_tokens,
            start_of_group_tokens,
            print_tokens,
            declaration_tokens,
            assignment_tokens,
            identifier_tokens,
            block_tokens,
        }
    }
}

struct Parser<'k> {
    current_token: Option<token::Token>,
    last_location: Option<location::FileLocation>,
    reporter: &'k dyn reporter::Reporter,
    tokens: Peekable<IntoIter<token::Token>>,
}
///
/// Parser stores the current token.
/// The current token can be taken with `take_current_token`
/// `advance` will take the next available token and store it in `current_token`
/// `consume_matching_token` if the next token matches a given token `advance` and return `true` otherwise it returns `false` and does not `advance`.
/// `consume_token` will `advance` if the next token matches the requested token type and fail otherwise
/// `check_next_token` checks if the next token is of the requested type
/// `declaration` will try to synchronize to a semi-colon after a failure is detected
impl<'k> Parser<'k> {
    fn new(reporter: &'k dyn reporter::Reporter, tokens: LinkedList<token::Token>) -> Self {
        Parser {
            current_token: None,
            last_location: None,
            reporter,
            tokens: tokens.into_iter().peekable(),
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

    fn parse(&mut self) -> LinkedList<stmt::Stmt> {
        let data = Data::new();

        let mut statements = LinkedList::new();
        while !self.is_at_end() {
            match self.declaration(&data) {
                Ok(stmt) => statements.push_back(stmt),
                Err(err) => self.reporter.add_message(&err.message),
            }
        }

        statements
    }

    fn declaration(&mut self, data: &Data) -> Result<stmt::Stmt, ParseError> {
        let result = if self.consume_matching_token(&data.declaration_tokens) {
            self.variable_declaration(data)
        } else {
            self.statement(data)
        };
        if result.is_err() {
            self.synchronize();
        }
        result
    }

    fn variable_declaration(&mut self, data: &Data) -> Result<stmt::Stmt, ParseError> {
        self.consume_token(&token::TokenType::Identifier, "Expect a variable name")?;
        let name = self.take_current_token()?;

        let initialiser = if self.consume_matching_token(&data.assignment_tokens) {
            Some(self.expression(data)?)
        } else {
            None
        };
        self.consume_semicolon("Expect ';' after variable declaration")?;
        Ok(stmt::Stmt::Var { name, initialiser })
    }

    fn statement(&mut self, data: &Data) -> Result<stmt::Stmt, ParseError> {
        if self.consume_matching_token(&data.print_tokens) {
            self.print_statement(data)
        } else if self.consume_matching_token(&data.block_tokens) {
            self.block_statement(data)
        } else {
            self.expression_statement(data)
        }
    }

    fn print_statement(&mut self, data: &Data) -> Result<stmt::Stmt, ParseError> {
        let value = self.expression(data)?;
        self.consume_semicolon("Expect ';' after value")?;
        Ok(stmt::Stmt::Print { value })
    }

    fn block_statement(&mut self, data: &Data) -> Result<stmt::Stmt, ParseError> {
        let mut statements = LinkedList::new();

        while !self.check_next_token(&token::TokenType::RightBrace) && !self.is_at_end() {
            statements.push_back(self.declaration(data)?);
        }

        self.consume_token(&token::TokenType::RightBrace, "Expect '}' after block")?;

        Ok(stmt::Stmt::Block { statements })
    }

    fn expression_statement(&mut self, data: &Data) -> Result<stmt::Stmt, ParseError> {
        let expression = self.expression(data)?;
        self.consume_semicolon("Expect ';' after expression")?;
        Ok(stmt::Stmt::Expression { expression })
    }

    fn expression(&mut self, data: &Data) -> Result<expr::Expr, ParseError> {
        self.assignment(data)
    }

    fn assignment(&mut self, data: &Data) -> Result<expr::Expr, ParseError> {
        let expr = self.equality(data)?;

        if self.consume_matching_token(&data.assignment_tokens) {
            let value = self.assignment(data)?;

            if let expr::Expr::Variable { name } = expr {
                return Ok(expr::Expr::build_assign(name, value));
            }
            let _ = self.add_diagnostic("Invalid assignment target");
        }
        Ok(expr)
    }

    fn equality(&mut self, data: &Data) -> Result<expr::Expr, ParseError> {
        let mut expr = self.comparison(data)?;

        while self.consume_matching_token(&data.equality_tokens) {
            let operator = self.take_current_token()?;
            let right = self.comparison(data)?;
            expr = expr::Expr::build_binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn comparison(&mut self, data: &Data) -> Result<expr::Expr, ParseError> {
        let mut expr = self.term(data)?;

        while self.consume_matching_token(&data.comparison_tokens) {
            let operator = self.take_current_token()?;
            let right = self.term(data)?;
            expr = expr::Expr::build_binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn term(&mut self, data: &Data) -> Result<expr::Expr, ParseError> {
        let mut expr = self.factor(data)?;

        while self.consume_matching_token(&data.term_tokens) {
            let operator = self.take_current_token()?;
            let right = self.factor(data)?;
            expr = expr::Expr::build_binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn factor(&mut self, data: &Data) -> Result<expr::Expr, ParseError> {
        let mut expr = self.unary(data)?;

        while self.consume_matching_token(&data.factor_tokens) {
            let operator = self.take_current_token()?;
            let right = self.unary(data)?;
            expr = expr::Expr::build_binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn unary(&mut self, data: &Data) -> Result<expr::Expr, ParseError> {
        if self.consume_matching_token(&data.unary_tokens) {
            let operator = self.take_current_token()?;
            let right = self.unary(data)?;
            return Ok(expr::Expr::build_unary(operator, right));
        }
        self.primary(data)
    }

    fn primary(&mut self, data: &Data) -> Result<expr::Expr, ParseError> {
        if self.consume_matching_token(&data.primary_tokens) {
            return Ok(expr::Expr::build_literal(self.take_current_token()?));
        }

        if self.consume_matching_token(&data.identifier_tokens) {
            return Ok(expr::Expr::build_variable(self.take_current_token()?));
        }

        if self.consume_matching_token(&data.start_of_group_tokens) {
            let expr = self.expression(data)?;
            self.consume_token(&token::TokenType::RightParen, "expect ')' after expression")?;
            return Ok(expr::Expr::build_grouping(expr));
        }

        self.add_diagnostic("Primary expression expected")
    }

    fn consume_token(
        &mut self,
        token_to_consume: &token::TokenType,
        message: &str,
    ) -> Result<(), ParseError> {
        if self.check_next_token(token_to_consume) {
            self.advance();
            return Ok(());
        }

        self.add_diagnostic(message)?; // cause method to fail
        Ok(())
    }

    fn consume_semicolon(&mut self, message: &str) -> Result<(), ParseError> {
        self.consume_token(&token::TokenType::Semicolon, message)?;
        Ok(())
    }

    fn consume_matching_token(&mut self, token_types: &[token::TokenType]) -> bool {
        if token_types.iter().any(|t| self.check_next_token(t)) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn check_next_token(&mut self, type_to_check: &token::TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            match self.tokens.peek() {
                Some(t) => t.token_type == *type_to_check,
                None => false,
            }
        }
    }

    fn is_at_end(&mut self) -> bool {
        match self.tokens.peek() {
            Some(t) => t.token_type == token::TokenType::Eof,
            None => true,
        }
    }

    fn advance(&mut self) {
        self.current_token = self.tokens.next();
        if self.current_token.is_some() {
            self.last_location = Some(self.current_token.as_ref().unwrap().start);
        }
    }

    fn add_diagnostic(&mut self, message: &str) -> Result<expr::Expr, ParseError> {
        let location = self
            .get_nearby_location()
            .unwrap_or(location::FileLocation::new(0, 0));
        self.reporter.add_diagnostic(&location, &location, message);
        Err(ParseError {
            message: message.to_string(),
        })
    }

    fn get_nearby_location(&mut self) -> Option<location::FileLocation> {
        if let Some(location) = &self.last_location {
            return Some(*location);
        }
        if let Some(token) = self.tokens.peek() {
            return Some(token.start);
        }
        None
    }

    fn synchronize(&mut self) {
        // untested
        while !self.is_at_end() {
            if let Some(token) = self.tokens.peek() {
                if token.token_type == token::TokenType::Semicolon {
                    self.advance();
                    return;
                }
            }

            if let Some(t) = &self.current_token {
                match t.token_type {
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
            }

            self.advance();
        }
    }
}
#[cfg(test)]
mod test {
    use super::*;
    use crate::reporter::test::TestReporter;
    use crate::{ast_printer, scanner, Reporter};

    #[test]
    fn production_tests() {
        let reporter = TestReporter::build();

        let tests = vec![
            ("10 + 10;", "(+ (10) (10)) ;"),
            ("10 == 10;", "(== (10) (10)) ;"),
            ("\"a string\";", "(a string) ;"),
            ("\"a string\" + 10;", "(+ (a string) (10)) ;"),
            ("(\"a string\" + 10);", "(group (+ (a string) (10))) ;"),
            ("print 10 == 11;", "PRINT (== (10) (11)) ;"),
            (" 10 > 11;", "(> (10) (11)) ;"),
            (" 10 * 11;", "(* (10) (11)) ;"),
            ("!!10;", "(! (! (10))) ;"),
            ("var a = 10;", "VAR a = (10) ;"),
            ("var a;", "VAR a ;"),
            ("{ var a; } ", "{\nVAR a ;\n}"),
            ("a = 10 ;", "a = (10) ;"),
        ];

        for (src, expected_parse) in tests {
            reporter.reset();

            let tokens = scanner::scan_tokens(&reporter, src);
            let statements = parse(&reporter, tokens);

            let parse = if statements.front().is_some() {
                ast_printer::print_stmt(statements.front().unwrap())
            } else {
                "".to_string()
            };
            if statements.len() != 1 || parse != expected_parse || reporter.has_diagnostics() {
                reporter.print_contents();
            }
            assert_eq!(statements.len(), 1, "Unexpected statements for '{}'", src);
            assert_eq!(
                parse, expected_parse,
                "unexpected parse of '{}'; {} does not match {}",
                src, parse, expected_parse
            );
            assert!(
                !reporter.has_diagnostics(),
                "unexpected diagnostics for '{}'",
                src
            );
        }
    }

    #[test]
    fn errors() {
        let reporter = TestReporter::build();

        let tests = vec![
            ("/ \"10\"", "Primary expression expected"),
            ("( \"10\"", "expect ')' after expression"),
            ("print 10", "Expect ';' after value"),
            ("\"10\"", "Expect ';' after expression"),
            ("\"10\" = 10 ;", "Invalid assignment target"),
        ];

        for (src, expected_message) in tests {
            reporter.reset();
            let tokens = scanner::scan_tokens(&reporter, src);
            let _ = parse(&reporter, tokens);
            if reporter.diagnostics_len() != 1 {
                reporter.print_contents();
                panic!("Unexpected diagnostics for '{}'", src);
            }

            assert_eq!(
                reporter
                    .diagnostic_get(0)
                    .expect("missing diagnostic")
                    .message,
                expected_message,
                "Missing diagnostic for '{}'",
                src
            );
        }
    }
}
