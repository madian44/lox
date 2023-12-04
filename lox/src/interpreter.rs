use crate::{expr, location, lox_type, reporter, stmt, token};
use std::collections::LinkedList;

#[derive(Debug, PartialEq)]
struct RuntimeError {
    message: String,
}

pub fn interpret(reporter: &dyn reporter::Reporter, statements: LinkedList<stmt::Stmt>) {
    for statement in statements {
        if let Err(err) = evalute_stmt(reporter, &statement) {
            reporter.add_message(&err.message);
        }
    }
}

fn evalute_stmt(
    reporter: &dyn reporter::Reporter,
    statement: &stmt::Stmt,
) -> Result<(), RuntimeError> {
    match statement {
        stmt::Stmt::Print { value } => {
            let result = evalute_expr(reporter, value)?;
            reporter.add_message(&format!("[print] {}", result));
            Ok(())
        }
        stmt::Stmt::Expression { expression } => match evalute_expr(reporter, expression) {
            Ok(r) => {
                reporter.add_message(&format!("[interpreter] {r}"));
                Ok(())
            }
            Err(err) => Err(err),
        },
    }
}

fn evalute_expr(
    reporter: &dyn reporter::Reporter,
    expression: &expr::Expr,
) -> Result<lox_type::LoxType, RuntimeError> {
    match expression {
        expr::Expr::Literal { value } => evaluate_literal(reporter, expression, value),
        expr::Expr::Grouping { expression } => evalute_expr(reporter, expression),
        expr::Expr::Unary { operator, right } => {
            evalute_unary(reporter, expression, operator, right)
        }
        expr::Expr::Binary {
            left,
            operator,
            right,
        } => evalute_binary(reporter, expression, left, operator, right),
    }
}

fn get_start_location(expr: &expr::Expr) -> &location::FileLocation {
    match expr {
        expr::Expr::Literal { value } => &value.start,
        expr::Expr::Unary { operator, right: _ } => &operator.start,
        expr::Expr::Binary {
            left,
            operator: _,
            right: _,
        } => get_start_location(left),
        expr::Expr::Grouping { expression } => get_start_location(expression),
    }
}

fn get_end_location(expr: &expr::Expr) -> &location::FileLocation {
    match expr {
        expr::Expr::Literal { value } => &value.end,
        expr::Expr::Unary { operator: _, right } => get_end_location(right),
        expr::Expr::Binary {
            left: _,
            operator: _,
            right,
        } => get_end_location(right),
        expr::Expr::Grouping { expression } => get_end_location(expression),
    }
}

fn add_diagnostic(
    reporter: &dyn reporter::Reporter,
    expr: &expr::Expr,
    message: String,
) -> Result<lox_type::LoxType, RuntimeError> {
    reporter.add_diagnostic(get_start_location(expr), get_end_location(expr), &message);
    Err(RuntimeError { message })
}

fn evaluate_literal(
    reporter: &dyn reporter::Reporter,
    expr: &expr::Expr,
    token: &token::Token,
) -> Result<lox_type::LoxType, RuntimeError> {
    match &token.literal {
        token::Literal::Number(number) => Ok(lox_type::LoxType::Number(*number)),
        token::Literal::String(string) => Ok(lox_type::LoxType::String(string.to_string())),
        token::Literal::True => Ok(lox_type::LoxType::Boolean(true)),
        token::Literal::False => Ok(lox_type::LoxType::Boolean(false)),
        token::Literal::Nil => Ok(lox_type::LoxType::Nil),
        _ => add_diagnostic(reporter, expr, "Unhandled literal".to_string()),
    }
}

fn evalute_unary(
    reporter: &dyn reporter::Reporter,
    expression: &expr::Expr,
    operator: &token::Token,
    right: &expr::Expr,
) -> Result<lox_type::LoxType, RuntimeError> {
    let right = evalute_expr(reporter, right)?;
    match operator.token_type {
        token::TokenType::Minus => {
            let right = check_number_operand(reporter, expression, &right)?;
            Ok(lox_type::LoxType::Number(-1.0 * right))
        }
        token::TokenType::Bang => Ok(lox_type::LoxType::Boolean(!is_truthy(&right))),
        _ => add_diagnostic(reporter, expression, "Unsupported operand".to_string()),
    }
}

fn evalute_binary(
    reporter: &dyn reporter::Reporter,
    expression: &expr::Expr,
    left: &expr::Expr,
    operator: &token::Token,
    right: &expr::Expr,
) -> Result<lox_type::LoxType, RuntimeError> {
    let left = evalute_expr(reporter, left)?;
    let right = evalute_expr(reporter, right)?;

    if matches!(operator.token_type, token::TokenType::Plus) {
        if matches!(right, lox_type::LoxType::Number(_))
            && matches!(left, lox_type::LoxType::Number(_))
        {
            let right = check_number_operand(reporter, expression, &right)?;
            let left = check_number_operand(reporter, expression, &left)?;
            Ok(lox_type::LoxType::Number(left + right))
        } else if matches!(right, lox_type::LoxType::String(_))
            && matches!(left, lox_type::LoxType::String(_))
        {
            let right = check_string_operand(reporter, expression, &right)?;
            let left = check_string_operand(reporter, expression, &left)?;
            Ok(lox_type::LoxType::String(left.to_string() + right))
        } else {
            add_diagnostic(
                reporter,
                expression,
                "Operands must be two numbers or two strings".to_string(),
            )
        }
    } else if matches!(operator.token_type, token::TokenType::EqualEqual) {
        Ok(lox_type::LoxType::Boolean(is_equal(&left, &right)))
    } else if matches!(operator.token_type, token::TokenType::BangEqual) {
        Ok(lox_type::LoxType::Boolean(!is_equal(&left, &right)))
    } else {
        let right = check_number_operand(reporter, expression, &right)?;
        let left = check_number_operand(reporter, expression, &left)?;
        match operator.token_type {
            token::TokenType::Minus => Ok(lox_type::LoxType::Number(left - right)),
            token::TokenType::Slash => Ok(lox_type::LoxType::Number(left / right)),
            token::TokenType::Star => Ok(lox_type::LoxType::Number(left * right)),
            token::TokenType::Greater => Ok(lox_type::LoxType::Boolean(left > right)),
            token::TokenType::GreaterEqual => Ok(lox_type::LoxType::Boolean(left >= right)),
            token::TokenType::Less => Ok(lox_type::LoxType::Boolean(left < right)),
            token::TokenType::LessEqual => Ok(lox_type::LoxType::Boolean(left <= right)),
            _ => add_diagnostic(reporter, expression, "Unsupported operator".to_string()),
        }
    }
}

fn is_equal(left: &lox_type::LoxType, right: &lox_type::LoxType) -> bool {
    if let lox_type::LoxType::Boolean(left) = left {
        if let lox_type::LoxType::Boolean(right) = right {
            return *left == *right;
        }
    }

    if let lox_type::LoxType::String(left) = left {
        if let lox_type::LoxType::String(right) = right {
            return *left == *right;
        }
    }

    if let lox_type::LoxType::Nil = left {
        if let lox_type::LoxType::Nil = right {
            return true;
        }
    }

    if let lox_type::LoxType::Number(left) = left {
        if let lox_type::LoxType::Number(right) = right {
            return *left == *right;
        }
    }

    false
}

fn is_truthy(lox_type: &lox_type::LoxType) -> bool {
    match lox_type {
        lox_type::LoxType::Nil => false,
        lox_type::LoxType::Boolean(bool) => *bool,
        _ => true,
    }
}

fn check_number_operand(
    reporter: &dyn reporter::Reporter,
    expression: &expr::Expr,
    lox_type: &lox_type::LoxType,
) -> Result<f64, RuntimeError> {
    match lox_type {
        lox_type::LoxType::Number(value) => Ok(*value),
        _ => Err(add_diagnostic(
            reporter,
            expression,
            "Operand should be a number".to_string(),
        )
        .unwrap_err()),
    }
}

fn check_string_operand<'a>(
    reporter: &dyn reporter::Reporter,
    expression: &'a expr::Expr,
    lox_type: &'a lox_type::LoxType,
) -> Result<&'a str, RuntimeError> {
    match &lox_type {
        lox_type::LoxType::String(string) => Ok(string),
        _ => Err(add_diagnostic(
            reporter,
            expression,
            "Operand should be a string".to_string(),
        )
        .unwrap_err()),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::ast_printer;
    use crate::expr;
    use crate::location::FileLocation;
    use crate::lox_type;
    use crate::reporter::test::TestReporter;
    use crate::token;

    #[test]
    fn test_truthy_values() {
        let tests = vec![
            (lox_type::LoxType::Nil, false),
            (lox_type::LoxType::Number(0f64), true),
            (lox_type::LoxType::Number(1f64), true),
            (lox_type::LoxType::String(String::from("")), true),
            (
                lox_type::LoxType::String(String::from("hello, world")),
                true,
            ),
            (lox_type::LoxType::Boolean(false), false),
            (lox_type::LoxType::Boolean(true), true),
        ];

        for (value, expected_result) in &tests {
            assert_eq!(
                is_truthy(value),
                *expected_result,
                "unexpected result: {:?} != {}",
                value,
                *expected_result
            );
        }
    }

    #[test]
    fn test_check_string_operand() {
        let reporter = TestReporter::build();
        let blank_location = FileLocation::new(0, 0);
        let expression = expr::Expr::Literal {
            value: token::Token::new(
                token::TokenType::String,
                "\"\"",
                blank_location,
                blank_location,
                token::Literal::String("".to_string()),
            ),
        };

        let tests = vec![
            (
                lox_type::LoxType::String("".to_string()),
                Ok::<&str, RuntimeError>(""),
            ),
            (
                lox_type::LoxType::Number(0f64),
                Err::<&str, RuntimeError>(RuntimeError {
                    message: "Operand should be a string".to_string(),
                }),
            ),
            (
                lox_type::LoxType::Nil,
                Err::<&str, RuntimeError>(RuntimeError {
                    message: "Operand should be a string".to_string(),
                }),
            ),
            (
                lox_type::LoxType::Boolean(true),
                Err::<&str, RuntimeError>(RuntimeError {
                    message: "Operand should be a string".to_string(),
                }),
            ),
        ];

        for (value, expected_result) in &tests {
            assert_eq!(
                check_string_operand(&reporter, &expression, value),
                *expected_result,
                "unexpected result for: {:?}",
                value
            );
        }
    }

    #[test]
    fn test_check_number_operand() {
        let reporter = TestReporter::build();
        let blank_location = FileLocation::new(0, 0);
        let expression = expr::Expr::Literal {
            value: token::Token::new(
                token::TokenType::String,
                "\"\"",
                blank_location,
                blank_location,
                token::Literal::String("".to_string()),
            ),
        };

        let tests = vec![
            (
                lox_type::LoxType::String("".to_string()),
                Err::<f64, RuntimeError>(RuntimeError {
                    message: "Operand should be a number".to_string(),
                }),
            ),
            (
                lox_type::LoxType::Number(0f64),
                Ok::<f64, RuntimeError>(0f64),
            ),
            (
                lox_type::LoxType::Nil,
                Err::<f64, RuntimeError>(RuntimeError {
                    message: "Operand should be a number".to_string(),
                }),
            ),
            (
                lox_type::LoxType::Boolean(true),
                Err::<f64, RuntimeError>(RuntimeError {
                    message: "Operand should be a number".to_string(),
                }),
            ),
        ];

        for (value, expected_result) in &tests {
            assert_eq!(
                check_number_operand(&reporter, &expression, value),
                *expected_result,
                "unexpected result for: {:?}",
                value
            );
        }
    }

    #[test]
    fn test_is_equal() {
        let tests = vec![
            (
                lox_type::LoxType::String("fred".to_string()),
                lox_type::LoxType::String("fred".to_string()),
                true,
            ),
            (
                lox_type::LoxType::String("".to_string()),
                lox_type::LoxType::String("fred".to_string()),
                false,
            ),
            (
                lox_type::LoxType::Number(10f64),
                lox_type::LoxType::Number(10f64),
                true,
            ),
            (
                lox_type::LoxType::Number(0f64),
                lox_type::LoxType::Number(10f64),
                false,
            ),
            (
                lox_type::LoxType::Number(0f64),
                lox_type::LoxType::Nil,
                false,
            ),
            (
                lox_type::LoxType::Boolean(true),
                lox_type::LoxType::Boolean(true),
                true,
            ),
            (
                lox_type::LoxType::Boolean(false),
                lox_type::LoxType::Boolean(false),
                true,
            ),
            (
                lox_type::LoxType::Boolean(false),
                lox_type::LoxType::Boolean(true),
                false,
            ),
            (
                lox_type::LoxType::Number(0f64),
                lox_type::LoxType::Boolean(true),
                false,
            ),
        ];

        for (left, right, expected_result) in &tests {
            assert_eq!(
                is_equal(left, right),
                *expected_result,
                "unexpected result {:?} == {:?}",
                left,
                right
            );
        }
    }

    #[test]
    fn test_plus() {
        let reporter = TestReporter::build();
        let blank_location = FileLocation::new(0, 0);
        let tests = vec![
            (
                expr::Expr::build_binary(
                    expr::Expr::Literal {
                        value: token::Token::new(
                            token::TokenType::Number,
                            "10",
                            blank_location,
                            blank_location,
                            token::Literal::Number(10f64),
                        ),
                    },
                    token::Token::new(
                        token::TokenType::Plus,
                        "+",
                        blank_location,
                        blank_location,
                        token::Literal::None,
                    ),
                    expr::Expr::Literal {
                        value: token::Token::new(
                            token::TokenType::Number,
                            "10",
                            blank_location,
                            blank_location,
                            token::Literal::Number(10f64),
                        ),
                    },
                ),
                Ok::<lox_type::LoxType, RuntimeError>(lox_type::LoxType::Number(20f64)),
            ),
            (
                expr::Expr::build_binary(
                    expr::Expr::Literal {
                        value: token::Token::new(
                            token::TokenType::String,
                            "\"hello,\"",
                            blank_location,
                            blank_location,
                            token::Literal::String("hello,".to_string()),
                        ),
                    },
                    token::Token::new(
                        token::TokenType::Plus,
                        "+",
                        blank_location,
                        blank_location,
                        token::Literal::None,
                    ),
                    expr::Expr::Literal {
                        value: token::Token::new(
                            token::TokenType::String,
                            "\" world\"",
                            blank_location,
                            blank_location,
                            token::Literal::String(" world".to_string()),
                        ),
                    },
                ),
                Ok::<lox_type::LoxType, RuntimeError>(lox_type::LoxType::String(
                    "hello, world".to_string(),
                )),
            ),
            (
                expr::Expr::build_binary(
                    expr::Expr::Literal {
                        value: token::Token::new(
                            token::TokenType::Number,
                            "\"10,\"",
                            blank_location,
                            blank_location,
                            token::Literal::Number(10f64),
                        ),
                    },
                    token::Token::new(
                        token::TokenType::Plus,
                        "+",
                        blank_location,
                        blank_location,
                        token::Literal::None,
                    ),
                    expr::Expr::Literal {
                        value: token::Token::new(
                            token::TokenType::String,
                            "\" world\"",
                            blank_location,
                            blank_location,
                            token::Literal::String(" world".to_string()),
                        ),
                    },
                ),
                Err::<lox_type::LoxType, RuntimeError>(RuntimeError {
                    message: "Operands must be two numbers or two strings".to_string(),
                }),
            ),
        ];

        for (expr, expected_result) in &tests {
            assert_eq!(
                evalute_expr(&reporter, expr),
                *expected_result,
                "unexpected result: {} != {:?}",
                ast_printer::print_expr(expr),
                expected_result
            );
        }
    }

    #[test]
    fn test_equality() {
        let reporter = TestReporter::build();
        let blank_location = FileLocation::new(0, 0);
        let tests = vec![
            (
                expr::Expr::build_binary(
                    expr::Expr::Literal {
                        value: token::Token::new(
                            token::TokenType::Number,
                            "10",
                            blank_location,
                            blank_location,
                            token::Literal::Number(10f64),
                        ),
                    },
                    token::Token::new(
                        token::TokenType::EqualEqual,
                        "==",
                        blank_location,
                        blank_location,
                        token::Literal::None,
                    ),
                    expr::Expr::Literal {
                        value: token::Token::new(
                            token::TokenType::Number,
                            "10",
                            blank_location,
                            blank_location,
                            token::Literal::Number(10f64),
                        ),
                    },
                ),
                Ok::<lox_type::LoxType, RuntimeError>(lox_type::LoxType::Boolean(true)),
            ),
            (
                expr::Expr::build_binary(
                    expr::Expr::Literal {
                        value: token::Token::new(
                            token::TokenType::Number,
                            "10",
                            blank_location,
                            blank_location,
                            token::Literal::Number(10f64),
                        ),
                    },
                    token::Token::new(
                        token::TokenType::BangEqual,
                        "!=",
                        blank_location,
                        blank_location,
                        token::Literal::None,
                    ),
                    expr::Expr::Literal {
                        value: token::Token::new(
                            token::TokenType::Number,
                            "10",
                            blank_location,
                            blank_location,
                            token::Literal::Number(10f64),
                        ),
                    },
                ),
                Ok::<lox_type::LoxType, RuntimeError>(lox_type::LoxType::Boolean(false)),
            ),
        ];

        for (expr, expected_result) in &tests {
            assert_eq!(
                evalute_expr(&reporter, expr),
                *expected_result,
                "unexpected result: {} != {:?}",
                ast_printer::print_expr(expr),
                expected_result
            );
        }
    }

    #[test]
    fn test_binary() {
        let reporter = TestReporter::build();
        let blank_location = FileLocation::new(0, 0);
        let tests = vec![
            (
                expr::Expr::build_binary(
                    expr::Expr::Literal {
                        value: token::Token::new(
                            token::TokenType::Number,
                            "10",
                            blank_location,
                            blank_location,
                            token::Literal::Number(10f64),
                        ),
                    },
                    token::Token::new(
                        token::TokenType::Minus,
                        "-",
                        blank_location,
                        blank_location,
                        token::Literal::None,
                    ),
                    expr::Expr::Literal {
                        value: token::Token::new(
                            token::TokenType::Number,
                            "5",
                            blank_location,
                            blank_location,
                            token::Literal::Number(5f64),
                        ),
                    },
                ),
                Ok::<lox_type::LoxType, RuntimeError>(lox_type::LoxType::Number(5f64)),
            ),
            (
                expr::Expr::build_binary(
                    expr::Expr::Literal {
                        value: token::Token::new(
                            token::TokenType::Number,
                            "10",
                            blank_location,
                            blank_location,
                            token::Literal::Number(10f64),
                        ),
                    },
                    token::Token::new(
                        token::TokenType::Star,
                        "*",
                        blank_location,
                        blank_location,
                        token::Literal::None,
                    ),
                    expr::Expr::Literal {
                        value: token::Token::new(
                            token::TokenType::Number,
                            "10",
                            blank_location,
                            blank_location,
                            token::Literal::Number(10f64),
                        ),
                    },
                ),
                Ok::<lox_type::LoxType, RuntimeError>(lox_type::LoxType::Number(100f64)),
            ),
        ];

        for (expr, expected_result) in &tests {
            assert_eq!(
                evalute_expr(&reporter, expr),
                *expected_result,
                "unexpected result: {} != {:?}",
                ast_printer::print_expr(expr),
                expected_result
            );
        }
    }

    #[test]
    fn test_unary() {
        let reporter = TestReporter::build();
        let blank_location = FileLocation::new(0, 0);
        let tests = vec![
            (
                expr::Expr::build_unary(
                    token::Token::new(
                        token::TokenType::Minus,
                        "-",
                        blank_location,
                        blank_location,
                        token::Literal::None,
                    ),
                    expr::Expr::Literal {
                        value: token::Token::new(
                            token::TokenType::Number,
                            "5",
                            blank_location,
                            blank_location,
                            token::Literal::Number(5f64),
                        ),
                    },
                ),
                Ok::<lox_type::LoxType, RuntimeError>(lox_type::LoxType::Number(-5f64)),
            ),
            (
                expr::Expr::build_unary(
                    token::Token::new(
                        token::TokenType::Bang,
                        "!",
                        blank_location,
                        blank_location,
                        token::Literal::None,
                    ),
                    expr::Expr::Literal {
                        value: token::Token::new(
                            token::TokenType::True,
                            "true",
                            blank_location,
                            blank_location,
                            token::Literal::True,
                        ),
                    },
                ),
                Ok::<lox_type::LoxType, RuntimeError>(lox_type::LoxType::Boolean(false)),
            ),
        ];

        for (expr, expected_result) in &tests {
            assert_eq!(
                evalute_expr(&reporter, expr),
                *expected_result,
                "unexpected result: {} != {:?}",
                ast_printer::print_expr(expr),
                expected_result
            );
        }
    }

    #[test]
    fn test_literal() {
        let reporter = TestReporter::build();
        let blank_location = FileLocation::new(0, 0);
        let tests = vec![
            (
                expr::Expr::build_literal(token::Token::new(
                    token::TokenType::String,
                    "\"hello, world\"",
                    blank_location,
                    blank_location,
                    token::Literal::String("hello, world".to_string()),
                )),
                Ok::<lox_type::LoxType, RuntimeError>(lox_type::LoxType::String(
                    "hello, world".to_string(),
                )),
            ),
            (
                expr::Expr::build_literal(token::Token::new(
                    token::TokenType::True,
                    "true",
                    blank_location,
                    blank_location,
                    token::Literal::True,
                )),
                Ok::<lox_type::LoxType, RuntimeError>(lox_type::LoxType::Boolean(true)),
            ),
        ];

        for (expr, expected_result) in &tests {
            assert_eq!(
                evalute_expr(&reporter, expr),
                *expected_result,
                "unexpected result: {} != {:?}",
                ast_printer::print_expr(expr),
                expected_result
            );
        }
    }
}
