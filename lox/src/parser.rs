use crate::{expr, location, reporter, stmt, token, FileLocation};
use std::collections::linked_list::IntoIter;
use std::collections::LinkedList;
use std::iter::Peekable;

const MAX_NUMBER_OF_ARGUMENTS: usize = 255;

#[derive(Debug)]
pub struct ParseError {
    message: String,
}

pub fn parse(
    reporter: &dyn reporter::Reporter,
    tokens: LinkedList<token::Token>,
) -> LinkedList<stmt::Stmt> {
    let mut parser = Parser::new(reporter, tokens, false);

    parser.parse()
}

pub fn parse_allow_invalid_call(
    reporter: &dyn reporter::Reporter,
    tokens: LinkedList<token::Token>,
) -> LinkedList<stmt::Stmt> {
    let mut parser = Parser::new(reporter, tokens, true);

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
    allow_invalid_call: bool,
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
    fn new(
        reporter: &'k dyn reporter::Reporter,
        tokens: LinkedList<token::Token>,
        allow_invalid_call: bool,
    ) -> Self {
        Parser {
            current_token: None,
            last_location: None,
            reporter,
            tokens: tokens.into_iter().peekable(),
            allow_invalid_call,
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
        let result = if self.consume_matching_token(&token::TokenType::Class) {
            self.class_declaration(data)
        } else if self.consume_matching_token(&token::TokenType::Fun) {
            self.function_declaration(data, "function")
        } else if self.consume_matching_token(&token::TokenType::Var) {
            self.variable_declaration(data)
        } else {
            self.statement(data)
        };
        if result.is_err() {
            self.synchronize();
        }
        result
    }

    fn class_declaration(&mut self, data: &Data) -> Result<stmt::Stmt, ParseError> {
        self.consume_token(&token::TokenType::Identifier, "Expect class name")?;
        let name = self.take_current_token()?;

        let superclass = if self.consume_matching_token(&token::TokenType::Less) {
            self.consume_token(&token::TokenType::Identifier, "Expect superclass name")?;
            Some(expr::Expr::new_variable(self.take_current_token()?))
        } else {
            None
        };

        self.consume_token(
            &token::TokenType::LeftBrace,
            "Expect '{{' before class body",
        )?;
        let mut methods = LinkedList::new();
        while !self.check_next_token(&token::TokenType::RightBrace) && !self.is_at_end() {
            methods.push_back(self.function_declaration(data, "method")?);
        }
        self.consume_token(
            &token::TokenType::RightBrace,
            "Expect '}}' after class body",
        )?;
        Ok(stmt::Stmt::Class {
            name,
            superclass,
            methods,
        })
    }

    fn function_declaration(&mut self, data: &Data, kind: &str) -> Result<stmt::Stmt, ParseError> {
        self.consume_token(
            &token::TokenType::Identifier,
            &format!("Expect {} name", kind),
        )?;
        let name = self.take_current_token()?;

        self.consume_token(
            &token::TokenType::LeftParen,
            &format!("Expect '(' after {} name", kind),
        )?;

        let mut params = LinkedList::new();
        if !self.check_next_token(&token::TokenType::RightParen) {
            loop {
                if params.len() > MAX_NUMBER_OF_ARGUMENTS {
                    // Just report the error
                    let _ = self.add_diagnostic(&format!(
                        "Cannot have more than {} parameters",
                        MAX_NUMBER_OF_ARGUMENTS
                    ));
                }

                self.consume_token(&token::TokenType::Identifier, "Expect parameter name")?;
                params.push_back(self.take_current_token()?);

                if !self.consume_matching_token(&token::TokenType::Comma) {
                    break;
                }
            }
        }
        self.consume_token(
            &token::TokenType::RightParen,
            &format!("Expect ')' after {} parameters", kind),
        )?;
        self.consume_token(
            &token::TokenType::LeftBrace,
            &format!("Expect '{{' before {} body", kind),
        )?;
        if let stmt::Stmt::Block { statements } = self.block_statement(data)? {
            Ok(stmt::Stmt::new_function(name, params, statements))
        } else {
            Err(ParseError {
                message: format!("Expect a block {} body", kind),
            })
        }
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
        } else if self.consume_matching_token(&token::TokenType::Return) {
            self.return_statement(data)
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

    fn return_statement(&mut self, data: &Data) -> Result<stmt::Stmt, ParseError> {
        let keyword = self.take_current_token()?;
        let value = if !self.check_next_token(&token::TokenType::Semicolon) {
            Some(self.expression(data)?)
        } else {
            None
        };
        self.consume_semicolon("Expect ';' after return value")?;
        Ok(stmt::Stmt::Return { keyword, value })
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
            expr::Expr::new_literal(token::Token::new(
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
        let error = self.consume_semicolon("Expect ';' after expression");
        if error.is_ok() {
            Ok(stmt::Stmt::Expression { expression })
        } else if !self.allow_invalid_call {
            Err(error.err().unwrap())
        } else {
            Ok(stmt::Stmt::Expression { expression })
        }
    }

    fn expression(&mut self, data: &Data) -> Result<expr::Expr, ParseError> {
        self.assignment_expression(data)
    }

    fn assignment_expression(&mut self, data: &Data) -> Result<expr::Expr, ParseError> {
        let expr = self.or_expression(data)?;

        if self.consume_matching_token(&token::TokenType::Equal) {
            let value = self.assignment_expression(data)?;

            if let expr::Expr::Variable { name, .. } = expr {
                return Ok(expr::Expr::new_assign(name, value));
            } else if let expr::Expr::Get { object, name, .. } = expr {
                return Ok(expr::Expr::new_set(*object, name, value));
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
            expr = expr::Expr::new_logical(expr, operator, right);
        }

        Ok(expr)
    }

    fn and_expression(&mut self, data: &Data) -> Result<expr::Expr, ParseError> {
        let mut expr = self.equality_expression(data)?;

        while self.consume_matching_token(&token::TokenType::And) {
            let operator = self.take_current_token()?;
            let right = self.equality_expression(data)?;
            expr = expr::Expr::new_logical(expr, operator, right);
        }

        Ok(expr)
    }

    fn equality_expression(&mut self, data: &Data) -> Result<expr::Expr, ParseError> {
        let mut expr = self.comparison_expression(data)?;

        while self.consume_any_matching_token(&data.equality_tokens) {
            let operator = self.take_current_token()?;
            let right = self.comparison_expression(data)?;
            expr = expr::Expr::new_binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn comparison_expression(&mut self, data: &Data) -> Result<expr::Expr, ParseError> {
        let mut expr = self.term_expression(data)?;

        while self.consume_any_matching_token(&data.comparison_tokens) {
            let operator = self.take_current_token()?;
            let right = self.term_expression(data)?;
            expr = expr::Expr::new_binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn term_expression(&mut self, data: &Data) -> Result<expr::Expr, ParseError> {
        let mut expr = self.factor_expression(data)?;

        while self.consume_any_matching_token(&data.term_tokens) {
            let operator = self.take_current_token()?;
            let right = self.factor_expression(data)?;
            expr = expr::Expr::new_binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn factor_expression(&mut self, data: &Data) -> Result<expr::Expr, ParseError> {
        let mut expr = self.unary_expression(data)?;

        while self.consume_any_matching_token(&data.factor_tokens) {
            let operator = self.take_current_token()?;
            let right = self.unary_expression(data)?;
            expr = expr::Expr::new_binary(expr, operator, right);
        }

        Ok(expr)
    }

    fn unary_expression(&mut self, data: &Data) -> Result<expr::Expr, ParseError> {
        if self.consume_any_matching_token(&data.unary_tokens) {
            let operator = self.take_current_token()?;
            let right = self.unary_expression(data)?;
            return Ok(expr::Expr::new_unary(operator, right));
        }
        self.call_expression(data)
    }

    fn call_expression(&mut self, data: &Data) -> Result<expr::Expr, ParseError> {
        let mut expr = self.primary_expression(data)?;

        loop {
            if self.consume_matching_token(&token::TokenType::LeftParen) {
                expr = self.finish_call(data, expr)?;
            } else if self.consume_matching_token(&token::TokenType::Dot) {
                let dot = self.take_current_token();
                let error = self.consume_token(
                    &token::TokenType::Identifier,
                    "Expect property name after '.'",
                );
                if error.is_ok() {
                    let name = self.take_current_token()?;
                    expr = expr::Expr::new_get(expr, name);
                } else if !self.allow_invalid_call {
                    return Err(error.err().unwrap());
                } else {
                    expr = expr::Expr::new_invalid_get(expr, dot.unwrap());
                }
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn finish_call(&mut self, data: &Data, callee: expr::Expr) -> Result<expr::Expr, ParseError> {
        let mut arguments = Vec::new();
        if !self.check_next_token(&token::TokenType::RightParen) {
            loop {
                if arguments.len() > MAX_NUMBER_OF_ARGUMENTS {
                    // Just report the error
                    let _ = self.add_diagnostic(&format!(
                        "Cannot have more than {} arguments",
                        MAX_NUMBER_OF_ARGUMENTS
                    ));
                }
                arguments.push(self.expression(data)?);
                if !self.check_next_token(&token::TokenType::Comma) {
                    break;
                }
                self.advance();
            }
        }
        self.consume_token(
            &token::TokenType::RightParen,
            "Expect ')' after function arguments",
        )?;
        let paren = self.take_current_token().unwrap();

        Ok(expr::Expr::new_call(callee, paren, arguments))
    }

    fn primary_expression(&mut self, data: &Data) -> Result<expr::Expr, ParseError> {
        if self.consume_any_matching_token(&data.primary_tokens) {
            return Ok(expr::Expr::new_literal(self.take_current_token()?));
        }

        if self.consume_matching_token(&token::TokenType::Super) {
            let keyword = self.take_current_token()?;
            self.consume_token(&token::TokenType::Dot, "Expect '.' after 'super'")?;
            let dot = self.take_current_token();

            let error = self.consume_token(
                &token::TokenType::Identifier,
                "Expect superclass method name",
            );

            if error.is_ok() {
                let method = self.take_current_token().unwrap();
                return Ok(expr::Expr::new_super(keyword, method));
            } else if !self.allow_invalid_call {
                return Err(error.err().unwrap());
            } else {
                return Ok(expr::Expr::new_invalid_super(keyword, dot.unwrap()));
            }
        }

        if self.consume_matching_token(&token::TokenType::This) {
            return Ok(expr::Expr::new_this(self.take_current_token()?));
        }

        if self.consume_matching_token(&token::TokenType::Identifier) {
            return Ok(expr::Expr::new_variable(self.take_current_token()?));
        }

        if self.consume_matching_token(&token::TokenType::LeftParen) {
            let expr = self.expression(data)?;
            self.consume_token(&token::TokenType::RightParen, "Expect ')' after expression")?;
            return Ok(expr::Expr::new_grouping(expr));
        }

        self.add_diagnostic("Expect expression")
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
        let reporter = TestReporter::new();

        let tests = vec![
            ("10 + 10;", "(; (+ 10 10))\n"),
            ("10 == 10;", "(; (== 10 10))\n"),
            ("\"a string\";", "(; \"a string\")\n"),
            ("\"a string\" + 10;", "(; (+ \"a string\" 10))\n"),
            ("(\"a string\" + 10);", "(; (group (+ \"a string\" 10)))\n"),
            ("print 10 == 11;", "(print (== 10 11))\n"),
            (" 10 > 11;", "(; (> 10 11))\n"),
            (" 10 * 11;", "(; (* 10 11))\n"),
            ("!!10;", "(; (! (! 10)))\n"),
            ("var a = 10;", "(var a = 10)\n"),
            ("var a;", "(var a)\n"),
            ("{ var a; } ", "(block\n    (var a)\n)\n"),
            ("a = 10 ;", "(; (= a 10))\n"),
            (
                "if ( a == 10 ) a = 10;",
                "(if (== a 10)\n    (; (= a 10))\n)\n",
            ),
            (
                "if ( a == 10 ) a = 10 ; else b = 20;",
                "(if-else (== a 10)\n    (; (= a 10))\n    (; (= b 20))\n)\n",
            ),
            (
                "if ( a == 10 or b == 20 ) a = 10;",
                "(if (or (== a 10) (== b 20))\n    (; (= a 10))\n)\n",
            ),
            (
                "if ( a == 10 and b == 20 ) a = 10;",
                "(if (and (== a 10) (== b 20))\n    (; (= a 10))\n)\n",
            ),
            (
                "while ( a == true ) a = false;",
                "(while (== a true)\n    (; (= a false))\n)\n",
            ),
            (
                "for ( var i = 1 ; i < 10 ; i = i + 1 ) print i;",
                "(block
                |    (var i = 1)
                |    (while (< i 10)
                |        (block
                |            (print i)
                |            (; (= i (+ i 1)))
                |        )
                |    )
                |)\n",
            ),
            (
                "for ( i = 1 ; true ; i = i + 1 ) print i;",
                "(block
                |    (; (= i 1))
                |    (while true
                |        (block
                |            (print i)
                |            (; (= i (+ i 1)))
                |        )
                |    )
                |)\n",
            ),
            (
                "for ( ; true ; i = i + 1 ) print i;",
                "(while true
                |    (block
                |        (print i)
                |        (; (= i (+ i 1)))
                |    )
                |)\n",
            ),
            (
                "for ( i = 1 ; true ; ) print i;",
                "(block
                |    (; (= i 1))
                |    (while true
                |        (print i)
                |    )
                |)\n",
            ),
            (
                "fun callee(a, b) { print a; print b ; }",
                "(fun callee(a b)
                |    (print a)
                |    (print b)
                |)\n",
            ),
            (
                "fun callee() { print c; print d ; }",
                "(fun callee()
                |    (print c)
                |    (print d)
                |)\n",
            ),
            (
                "class a_class { method_1() {} method_2(a) {}}",
                "(class a_class
                |    (fun method_1()
                |    )
                |    (fun method_2(a)
                |    )
                |)\n",
            ),
            (
                "fun callee() { return 10; }",
                "(fun callee()
                |    (return 10)
                |)\n",
            ),
            (
                "class a_class { init() {this.value = 10;}}",
                "(class a_class
                |    (fun init()
                |        (; (= this value 10)
                |    )
                |)\n",
            ),
            (
                "class sub_class < super_class {}",
                "(class sub_class < super_class
                |)\n",
            ),
            ("super.method ; ", "(; (super method))\n"),
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
        let reporter = TestReporter::new();

        let tests = vec![
            ("/ \"10\"", "Expect expression"),
            ("( \"10\"", "Expect ')' after expression"),
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
            ("callee ( ;", "Expect expression"),
            ("callee ( a ;", "Expect ')' after function arguments"),
            ("callee ( a, ;", "Expect expression"),
            ("fun ( ;", "Expect function name"),
            ("fun callee a ;", "Expect '(' after function name"),
            ("fun callee ( 10 ;", "Expect parameter name"),
            ("fun callee ( a ;", "Expect ')' after function parameters"),
            (
                "fun callee ( a ) print a;",
                "Expect '{' before function body",
            ),
            ("class { method_1() {} method_2(a) {}}", "Expect class name"),
            (
                "class a_class method_1() {} method_2(a) {}}",
                "Expect '{{' before class body",
            ),
            (
                "class a_class {method_1() {} method_2(a) {}",
                "Expect '}}' after class body",
            ),
            ("class sub_class < {}", "Expect superclass name"),
            ("super 10", "Expect '.' after 'super'"),
            ("super", "Expect '.' after 'super'"),
            ("super.10", "Expect superclass method name"),
            ("fred.10", "Expect property name after '.'"),
        ];

        for (src, expected_message) in tests {
            reporter.reset();
            let tokens = scanner::scan_tokens(&reporter, src);
            let _ = parse(&reporter, tokens);
            if reporter.diagnostics_len() == 0 {
                reporter.print_contents();
                panic!("Unexpectedly no diagnostics for '{}'", src);
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

    #[test]
    fn allow_invalid_call_tests() {
        let reporter = TestReporter::new();

        let tests = vec![
            ("test. ;", "(; (test..))\n"),
            ("super. ;", "(; (super .))\n"),
        ];

        for (src, expected_parse) in tests {
            reporter.reset();
            let expected_parse = unindent_string(expected_parse);

            let tokens = scanner::scan_tokens(&reporter, src);
            let statements = parse_allow_invalid_call(&reporter, tokens);

            let parse = if statements.front().is_some() {
                ast_printer::print_stmt(statements.front().unwrap())
            } else {
                "".to_string()
            };
            if statements.len() != 1 || parse != expected_parse {
                reporter.print_contents();
            }
            assert_eq!(statements.len(), 1, "Unexpected statements for '{}'", src);
            assert_eq!(
                parse, expected_parse,
                "unexpected parse of '{}'; {} does not match {}",
                src, parse, expected_parse
            );
        }
    }
}
