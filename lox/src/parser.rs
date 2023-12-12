use crate::{expr, location, reporter, stmt, token, FileLocation};
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
        Data {
            equality_tokens,
            comparison_tokens,
            factor_tokens,
            term_tokens,
            unary_tokens,
            primary_tokens,
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
/// `consume_any_matching_token` if the next token matches a given token `advance` and return `true` otherwise it returns `false` and does not `advance`.
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
        let result = if self.consume_matching_token(&token::TokenType::Var) {
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

        let initialiser = if self.consume_matching_token(&token::TokenType::Equal) {
            Some(self.expression(data)?)
        } else {
            None
        };
        self.consume_semicolon("Expect ';' after variable declaration")?;
        Ok(stmt::Stmt::Var { name, initialiser })
    }

    fn statement(&mut self, data: &Data) -> Result<stmt::Stmt, ParseError> {
        if self.consume_matching_token(&token::TokenType::For) {
            self.for_statement(data)
        } else if self.consume_matching_token(&token::TokenType::If) {
            self.if_statement(data)
        } else if self.consume_matching_token(&token::TokenType::Print) {
            self.print_statement(data)
        } else if self.consume_matching_token(&token::TokenType::While) {
            self.while_statement(data)
        } else if self.consume_matching_token(&token::TokenType::LeftBrace) {
            self.block_statement(data)
        } else {
            self.expression_statement(data)
        }
    }

    fn if_statement(&mut self, data: &Data) -> Result<stmt::Stmt, ParseError> {
        self.consume_token(&token::TokenType::LeftParen, "Expect '(' after 'if'")?;
        let condition = self.expression(data)?;
        self.consume_token(
            &token::TokenType::RightParen,
            "Expect ')' after 'if' condition",
        )?;

        let then_branch = self.statement(data)?;

        let mut else_branch = None;
        if self.consume_matching_token(&token::TokenType::Else) {
            else_branch = Some(self.statement(data)?);
        }
        Ok(stmt::Stmt::If {
            condition,
            then_branch: Box::new(then_branch),
            else_branch: Box::new(else_branch),
        })
    }

    fn print_statement(&mut self, data: &Data) -> Result<stmt::Stmt, ParseError> {
        let value = self.expression(data)?;
        self.consume_semicolon("Expect ';' after value")?;
        Ok(stmt::Stmt::Print { value })
    }

    fn while_statement(&mut self, data: &Data) -> Result<stmt::Stmt, ParseError> {
        self.consume_token(&token::TokenType::LeftParen, "Expect '(' after 'while'")?;
        let condition = self.expression(data)?;
        self.consume_token(
            &token::TokenType::RightParen,
            "Expect ')' after 'while' condition",
        )?;
        let body = self.statement(data)?;
        Ok(stmt::Stmt::While {
            condition,
            body: Box::new(body),
        })
    }

    fn for_statement(&mut self, data: &Data) -> Result<stmt::Stmt, ParseError> {
        self.consume_token(&token::TokenType::LeftParen, "Expect '(' after 'for'")?;

        let initialiser = if self.consume_matching_token(&token::TokenType::Semicolon) {
            None
        } else if self.consume_matching_token(&token::TokenType::Var) {
            Some(self.variable_declaration(data)?)
        } else {
            Some(self.expression_statement(data)?)
        };

        let condition = if !self.check_next_token(&token::TokenType::Semicolon) {
            self.expression(data)?
        } else {
            expr::Expr::build_literal(token::Token::new(
                token::TokenType::True,
                "true",
                self.get_nearby_location()
                    .unwrap_or(FileLocation::new(0, 0)),
                self.get_nearby_location()
                    .unwrap_or(FileLocation::new(0, 0)),
                Some(token::Literal::True),
            ))
        };
        self.consume_token(
            &token::TokenType::Semicolon,
            "Expect ';' after 'for' loop condition",
        )?;

        let increment = if !self.check_next_token(&token::TokenType::RightParen) {
            Some(self.expression(data)?)
        } else {
            None
        };

        self.consume_token(
            &token::TokenType::RightParen,
            "Expect ')' after 'for' clauses",
        )?;

        let mut body = self.statement(data)?;
        body = if let Some(increment) = increment {
            let mut desugared_body = LinkedList::new();
            desugared_body.push_back(body);
            desugared_body.push_back(stmt::Stmt::Expression {
                expression: increment,
            });
            stmt::Stmt::Block {
                statements: desugared_body,
            }
        } else {
            body
        };
        body = stmt::Stmt::While {
            condition,
            body: Box::new(body),
        };

        body = if let Some(initialiser) = initialiser {
            let mut desugared_initialiser = LinkedList::new();
            desugared_initialiser.push_back(initialiser);
            desugared_initialiser.push_back(body);
            stmt::Stmt::Block {
                statements: desugared_initialiser,
            }
        } else {
            body
        };

        Ok(body)
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
        self.assignment_expression(data)
    }

    fn assignment_expression(&mut self, data: &Data) -> Result<expr::Expr, ParseError> {
        let expr = self.or_expression(data)?;

        if self.consume_matching_token(&token::TokenType::Equal) {
            let value = self.assignment_expression(data)?;

            if let expr::Expr::Variable { name } = expr {
                return Ok(expr::Expr::build_assign(name, value));
            }
            let _ = self.add_diagnostic("Invalid assignment target");
        }
        Ok(expr)
    }

    fn or_expression(&mut self, data: &Data) -> Result<expr::Expr, ParseError> {
        let mut expr = self.and_expression(data)?;

        while self.consume_matching_token(&token::TokenType::Or) {
            let operator = self.take_current_token()?;
            let right = self.and_expression(data)?;
            expr = expr::Expr::build_logical(expr, operator, right);
        }

        Ok(expr)
    }

    fn and_expression(&mut self, data: &Data) -> Result<expr::Expr, ParseError> {
        let mut expr = self.equality_expression(data)?;

        while self.consume_matching_token(&token::TokenType::And) {
            let operator = self.take_current_token()?;
            let right = self.equality_expression(data)?;
            expr = expr::Expr::build_logical(expr, operator, right);
        }

        Ok(expr)
    }

    fn equality_expression(&mut self, data: &Data) -> Result<expr::Expr, ParseError> {
        let mut expr = self.comparison_expression(data)?;

        while self.consume_any_matching_token(&data.equality_tokens) {
            let operator = self.take_current_token()?;
            let right = self.comparison_expression(data)?;
            expr = expr::Expr::build_binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn comparison_expression(&mut self, data: &Data) -> Result<expr::Expr, ParseError> {
        let mut expr = self.term_expression(data)?;

        while self.consume_any_matching_token(&data.comparison_tokens) {
            let operator = self.take_current_token()?;
            let right = self.term_expression(data)?;
            expr = expr::Expr::build_binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn term_expression(&mut self, data: &Data) -> Result<expr::Expr, ParseError> {
        let mut expr = self.factor_expression(data)?;

        while self.consume_any_matching_token(&data.term_tokens) {
            let operator = self.take_current_token()?;
            let right = self.factor_expression(data)?;
            expr = expr::Expr::build_binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn factor_expression(&mut self, data: &Data) -> Result<expr::Expr, ParseError> {
        let mut expr = self.unary_expression(data)?;

        while self.consume_any_matching_token(&data.factor_tokens) {
            let operator = self.take_current_token()?;
            let right = self.unary_expression(data)?;
            expr = expr::Expr::build_binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn unary_expression(&mut self, data: &Data) -> Result<expr::Expr, ParseError> {
        if self.consume_any_matching_token(&data.unary_tokens) {
            let operator = self.take_current_token()?;
            let right = self.unary_expression(data)?;
            return Ok(expr::Expr::build_unary(operator, right));
        }
        self.primary_expression(data)
    }

    fn primary_expression(&mut self, data: &Data) -> Result<expr::Expr, ParseError> {
        if self.consume_any_matching_token(&data.primary_tokens) {
            return Ok(expr::Expr::build_literal(self.take_current_token()?));
        }

        if self.consume_matching_token(&token::TokenType::Identifier) {
            return Ok(expr::Expr::build_variable(self.take_current_token()?));
        }

        if self.consume_matching_token(&token::TokenType::LeftParen) {
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

    fn consume_any_matching_token(&mut self, token_types: &[token::TokenType]) -> bool {
        if token_types.iter().any(|t| self.check_next_token(t)) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn consume_matching_token(&mut self, token_type: &token::TokenType) -> bool {
        if self.check_next_token(token_type) {
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

    fn unindent_string(source: &str) -> String {
        let re = regex::Regex::new(r"\n\s+[|]").unwrap();
        re.replace_all(source, "\n").to_string()
    }

    #[test]
    fn production_tests() {
        let reporter = TestReporter::build();

        let tests = vec![
            ("10 + 10;", "(expression (+ (10) (10)))\n"),
            ("10 == 10;", "(expression (== (10) (10)))\n"),
            ("\"a string\";", "(expression (\"a string\"))\n"),
            (
                "\"a string\" + 10;",
                "(expression (+ (\"a string\") (10)))\n",
            ),
            (
                "(\"a string\" + 10);",
                "(expression (group (+ (\"a string\") (10))))\n",
            ),
            ("print 10 == 11;", "(print (== (10) (11)))\n"),
            (" 10 > 11;", "(expression (> (10) (11)))\n"),
            (" 10 * 11;", "(expression (* (10) (11)))\n"),
            ("!!10;", "(expression (! (! (10))))\n"),
            ("var a = 10;", "(var (a) (10))\n"),
            ("var a;", "(var (a))\n"),
            ("{ var a; } ", "(block\n    (var (a))\n)\n"),
            ("a = 10 ;", "(expression (= (a) (10)))\n"),
            (
                "if ( a == 10 ) a = 10;",
                "(if (== (a) (10))\n    (expression (= (a) (10)))\n)\n",
            ),
            (
                "if ( a == 10 ) a = 10 ; else b = 20;",
                "(if (== (a) (10))\n    (expression (= (a) (10)))\n    (expression (= (b) (20)))\n)\n",
            ),
            (
                "if ( a == 10 or b == 20 ) a = 10;",
                "(if (or (== (a) (10)) (== (b) (20)))\n    (expression (= (a) (10)))\n)\n",
            ),
            (
                "if ( a == 10 and b == 20 ) a = 10;",
                "(if (and (== (a) (10)) (== (b) (20)))\n    (expression (= (a) (10)))\n)\n",
            ),
            (
                "while ( a == true ) a = false;",
                "(while (== (a) (True))\n    (expression (= (a) (False)))\n)\n",
            ),
            (
                "for ( var i = 1 ; i < 10 ; i = i + 1 ) print i;",
                "(block
                |    (var (i) (1))
                |    (while (< (i) (10))
                |        (block
                |            (print (i))
                |            (expression (= (i) (+ (i) (1))))
                |        )
                |    )
                |)\n",
            ),
            (
                "for ( i = 1 ; true ; i = i + 1 ) print i;",
                "(block
                |    (expression (= (i) (1)))
                |    (while (True)
                |        (block
                |            (print (i))
                |            (expression (= (i) (+ (i) (1))))
                |        )
                |    )
                |)\n",
            ),
            (
                "for ( ; true ; i = i + 1 ) print i;",
                "(while (True)
                |    (block
                |        (print (i))
                |        (expression (= (i) (+ (i) (1))))
                |    )
                |)\n",
            ),
            (
                "for ( i = 1 ; true ; ) print i;",
                "(block
                |    (expression (= (i) (1)))
                |    (while (True)
                |        (print (i))
                |    )
                |)\n",
            ),
        ];

        for (src, expected_parse) in tests {
            reporter.reset();
            let expected_parse = unindent_string(expected_parse);

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
            (
                "if ( a = 10 ) a = 10 ; else { b = 20;",
                "Expect '}' after block",
            ),
            ("if  a = 10 ) a = 10 ; ", "Expect '(' after 'if'"),
            ("if ( a = 10  a = 10 ; ", "Expect ')' after 'if' condition"),
            ("while  true ) a = 10 ; ", "Expect '(' after 'while'"),
            (
                "while ( true a = 10 ; ",
                "Expect ')' after 'while' condition",
            ),
            (
                "for  i = 1 ; ; i = i + 1 ) print i;",
                "Expect '(' after 'for'",
            ),
            (
                "for ( i = 1 ; i = i + 1 ) print i;",
                "Expect ';' after 'for' loop condition",
            ),
            (
                "for ( i = 1 ; ; i = i + 1  print i;",
                "Expect ')' after 'for' clauses",
            ),
        ];

        for (src, expected_message) in tests {
            reporter.reset();
            let tokens = scanner::scan_tokens(&reporter, src);
            let _ = parse(&reporter, tokens);
            if reporter.diagnostics_len() == 0 {
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
