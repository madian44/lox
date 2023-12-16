mod environment;
mod function;
mod lox_type;
mod native_functions;
mod unwind;

use crate::{expr, location, reporter, stmt, token};
use std::collections::LinkedList;
use std::rc;

pub fn interpret(reporter: &dyn reporter::Reporter, statements: LinkedList<stmt::Stmt>) {
    let mut environment = environment::Environment::new();
    Interpreter::define_native_functions(&mut environment);

    let _ = interpret_with_environment(reporter, &mut environment, &statements);
}

fn interpret_with_environment(
    reporter: &dyn reporter::Reporter,
    environment: &mut environment::Environment,
    statements: &LinkedList<stmt::Stmt>,
) -> Result<(), unwind::Unwind> {
    let mut interpreter = Interpreter::build(reporter);

    for statement in statements {
        match interpreter.evaluate_stmt(environment, statement) {
            Err(unwind::Unwind::WithError(message)) => {
                reporter.add_message(&message);
                return Err(unwind::Unwind::WithError(message));
            }
            Err(unwind::Unwind::WithResult(value)) => {
                return Err(unwind::Unwind::WithResult(value));
            }
            _ => (),
        }
    }
    Ok(())
}

struct Interpreter<'r> {
    reporter: &'r dyn reporter::Reporter,
    //    environment: environment::Environment,
}

impl<'r> Interpreter<'r> {
    fn build(reporter: &'r dyn reporter::Reporter) -> Self {
        Self { reporter }
    }

    fn define_native_functions(environment: &mut environment::Environment) {
        environment.define(
            "clock",
            lox_type::LoxType::NativeFunction {
                name: "clock".to_string(),
                callable: rc::Rc::new(Box::new(native_functions::Clock)),
            },
        );
    }

    fn evaluate_stmt(
        &mut self,
        environment: &mut environment::Environment,
        statement: &stmt::Stmt,
    ) -> Result<(), unwind::Unwind> {
        match statement {
            stmt::Stmt::Block { statements } => self.evaluate_stmt_block(environment, statements),
            stmt::Stmt::Expression { expression } => {
                self.evaluate_stmt_expression(environment, expression)
            }
            stmt::Stmt::Function { name, params, body } => {
                self.evaluate_stmt_function(environment, name, params, body)
            }
            stmt::Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => self.evaluate_stmt_if(environment, condition, then_branch, else_branch),
            stmt::Stmt::Print { value } => self.evaluate_stmt_print(environment, value),
            stmt::Stmt::Return { keyword, value } => {
                self.evaluate_stmt_return(environment, keyword, value)
            }
            stmt::Stmt::Var { name, initialiser } => {
                self.evaluate_stmt_var(environment, name, initialiser)
            }
            stmt::Stmt::While { condition, body } => {
                self.evaluate_stmt_while(environment, condition, body)
            }
        }
    }

    fn evaluate_stmt_block(
        &mut self,
        environment: &mut environment::Environment,
        statements: &LinkedList<stmt::Stmt>,
    ) -> Result<(), unwind::Unwind> {
        environment.new_frame();
        for statement in statements {
            if let Err(err) = self.evaluate_stmt(environment, statement) {
                environment.pop_frame();
                return Err(err);
            }
        }
        environment.pop_frame();
        Ok(())
    }

    fn evaluate_stmt_expression(
        &mut self,
        environment: &mut environment::Environment,
        expr: &expr::Expr,
    ) -> Result<(), unwind::Unwind> {
        match self.evaluate_expr(environment, expr) {
            Ok(r) => {
                self.reporter.add_message(&format!("[interpreter] {r}"));
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    fn evaluate_stmt_function(
        &mut self,
        environment: &mut environment::Environment,
        name: &token::Token,
        params: &LinkedList<token::Token>,
        body: &LinkedList<stmt::Stmt>,
    ) -> Result<(), unwind::Unwind> {
        let function_name = name.lexeme.clone();
        let function = function::Function::build(name.clone(), params.clone(), (*body).clone());
        let function = lox_type::LoxType::Function {
            name: function_name.clone(),
            callable: rc::Rc::new(Box::new(function)),
        };
        environment.define(&function_name, function);
        Ok(())
    }

    fn evaluate_stmt_if(
        &mut self,
        environment: &mut environment::Environment,
        condition: &expr::Expr,
        then_branch: &stmt::Stmt,
        else_branch: &Option<stmt::Stmt>,
    ) -> Result<(), unwind::Unwind> {
        if is_truthy(&self.evaluate_expr(environment, condition)?) {
            self.evaluate_stmt(environment, then_branch)?;
        } else if let Some(else_branch) = else_branch {
            self.evaluate_stmt(environment, else_branch)?;
        }
        Ok(())
    }

    fn evaluate_stmt_print(
        &mut self,
        environment: &mut environment::Environment,
        expr: &expr::Expr,
    ) -> Result<(), unwind::Unwind> {
        let result = self.evaluate_expr(environment, expr)?;
        self.reporter.add_message(&format!("[print] {}", result));
        Ok(())
    }

    fn evaluate_stmt_return(
        &mut self,
        environment: &mut environment::Environment,
        _: &token::Token,
        value: &Option<expr::Expr>,
    ) -> Result<(), unwind::Unwind> {
        let result = if let Some(value) = value {
            self.evaluate_expr(environment, value)?
        } else {
            lox_type::LoxType::Nil
        };

        Err(unwind::Unwind::WithResult(result))
    }

    fn evaluate_stmt_var(
        &mut self,
        environment: &mut environment::Environment,
        name: &token::Token,
        initialiser: &Option<expr::Expr>,
    ) -> Result<(), unwind::Unwind> {
        let initial_value = match initialiser {
            Some(expression) => self.evaluate_expr(environment, expression)?,
            None => lox_type::LoxType::Nil,
        };
        environment.define(&name.lexeme, initial_value);
        Ok(())
    }

    fn evaluate_stmt_while(
        &mut self,
        environment: &mut environment::Environment,
        condition: &expr::Expr,
        body: &stmt::Stmt,
    ) -> Result<(), unwind::Unwind> {
        while is_truthy(&(self.evaluate_expr(environment, condition)?)) {
            self.evaluate_stmt(environment, body)?;
        }
        Ok(())
    }

    fn evaluate_expr(
        &mut self,
        environment: &mut environment::Environment,
        expression: &expr::Expr,
    ) -> Result<lox_type::LoxType, unwind::Unwind> {
        match expression {
            expr::Expr::Assign { name, value } => {
                self.evaluate_expr_assign(environment, expression, name, value)
            }
            expr::Expr::Binary {
                left,
                operator,
                right,
            } => self.evaluate_expr_binary(environment, expression, left, operator, right),
            expr::Expr::Call {
                callee,
                paren,
                arguments,
            } => self.evaluate_expr_call(environment, callee, paren, arguments),
            expr::Expr::Grouping { expression } => self.evaluate_expr(environment, expression),
            expr::Expr::Literal { value } => self.evaluate_expr_literal(expression, value),
            expr::Expr::Logical {
                left,
                operator,
                right,
            } => self.evaluate_expr_logical(environment, left, operator, right),
            expr::Expr::Unary { operator, right } => {
                self.evaluate_expr_unary(environment, expression, operator, right)
            }
            expr::Expr::Variable { name } => self.evaluate_expr_var(environment, expression, name),
        }
    }

    fn evaluate_expr_assign(
        &mut self,
        environment: &mut environment::Environment,
        expression: &expr::Expr,
        name: &token::Token,
        value: &expr::Expr,
    ) -> Result<lox_type::LoxType, unwind::Unwind> {
        let value = self.evaluate_expr(environment, value)?;
        if let Err(unwind::Unwind::WithError(message)) = environment.assign(name, value.clone()) {
            self.add_diagnostic(expression, message)?;
        }
        Ok(value)
    }

    fn evaluate_expr_binary(
        &mut self,
        environment: &mut environment::Environment,
        expression: &expr::Expr,
        left: &expr::Expr,
        operator: &token::Token,
        right: &expr::Expr,
    ) -> Result<lox_type::LoxType, unwind::Unwind> {
        let left = self.evaluate_expr(environment, left)?;
        let right = self.evaluate_expr(environment, right)?;

        if matches!(operator.token_type, token::TokenType::Plus) {
            if matches!(right, lox_type::LoxType::Number(_))
                && matches!(left, lox_type::LoxType::Number(_))
            {
                let right = self.check_number_operand(expression, &right)?;
                let left = self.check_number_operand(expression, &left)?;
                Ok(lox_type::LoxType::Number(left + right))
            } else if matches!(right, lox_type::LoxType::String(_))
                && matches!(left, lox_type::LoxType::String(_))
            {
                let right = self.check_string_operand(expression, &right)?;
                let left = self.check_string_operand(expression, &left)?;
                Ok(lox_type::LoxType::String(left.to_string() + right))
            } else {
                self.add_diagnostic(
                    expression,
                    "Operands must be two numbers or two strings".to_string(),
                )
            }
        } else if matches!(operator.token_type, token::TokenType::EqualEqual) {
            Ok(lox_type::LoxType::Boolean(is_equal(&left, &right)))
        } else if matches!(operator.token_type, token::TokenType::BangEqual) {
            Ok(lox_type::LoxType::Boolean(!is_equal(&left, &right)))
        } else {
            let right = self.check_number_operand(expression, &right)?;
            let left = self.check_number_operand(expression, &left)?;
            match operator.token_type {
                token::TokenType::Minus => Ok(lox_type::LoxType::Number(left - right)),
                token::TokenType::Slash => Ok(lox_type::LoxType::Number(left / right)),
                token::TokenType::Star => Ok(lox_type::LoxType::Number(left * right)),
                token::TokenType::Greater => Ok(lox_type::LoxType::Boolean(left > right)),
                token::TokenType::GreaterEqual => Ok(lox_type::LoxType::Boolean(left >= right)),
                token::TokenType::Less => Ok(lox_type::LoxType::Boolean(left < right)),
                token::TokenType::LessEqual => Ok(lox_type::LoxType::Boolean(left <= right)),
                _ => self.add_diagnostic(expression, "Unsupported operator".to_string()),
            }
        }
    }

    fn evaluate_expr_call(
        &mut self,
        environment: &mut environment::Environment,
        callee: &expr::Expr,
        _: &token::Token,
        arguments: &Vec<expr::Expr>,
    ) -> Result<lox_type::LoxType, unwind::Unwind> {
        let actual_callee = self.evaluate_expr(environment, callee)?;

        let mut args = Vec::new();
        for expr in arguments {
            args.push(self.evaluate_expr(environment, expr)?);
        }
        let result = self.call_function(environment, actual_callee, callee, args);
        match result {
            Err(unwind::Unwind::WithError(_)) => result,
            Ok(value) => Ok(value),
            _ => unreachable!(),
        }
    }

    fn call_function(
        &mut self,
        environment: &mut environment::Environment,
        callee: lox_type::LoxType,
        expr: &expr::Expr,
        arguments: Vec<lox_type::LoxType>,
    ) -> Result<lox_type::LoxType, unwind::Unwind> {
        let check_arity = |callable_arity: usize| -> Result<lox_type::LoxType, unwind::Unwind> {
            if arguments.len() != callable_arity {
                self.add_diagnostic(
                    expr,
                    format!(
                        "Expected {} arguments but got {}",
                        callable_arity,
                        arguments.len()
                    ),
                )?;
            }
            Ok(lox_type::LoxType::Nil)
        };
        match callee {
            lox_type::LoxType::Function { name: _, callable } => {
                check_arity(callable.arity())?;
                match callable.call(self.reporter, environment, arguments) {
                    Err(unwind::Unwind::WithError(message)) => {
                        Err(unwind::Unwind::WithError(message))
                    }
                    Err(unwind::Unwind::WithResult(value)) => Ok(value),
                    _ => Ok(lox_type::LoxType::Nil),
                }
            }
            lox_type::LoxType::NativeFunction { name: _, callable } => {
                check_arity(callable.arity())?;
                callable.call(arguments)
            }
            _ => Err(self
                .add_diagnostic(expr, "Can only call functions and classes".to_string())
                .unwrap_err()),
        }
    }

    fn evaluate_expr_literal(
        &self,
        expr: &expr::Expr,
        token: &token::Token,
    ) -> Result<lox_type::LoxType, unwind::Unwind> {
        match &token.literal {
            Some(token::Literal::Number(number)) => Ok(lox_type::LoxType::Number(*number)),
            Some(token::Literal::String(string)) => {
                Ok(lox_type::LoxType::String(string.to_string()))
            }
            Some(token::Literal::True) => Ok(lox_type::LoxType::Boolean(true)),
            Some(token::Literal::False) => Ok(lox_type::LoxType::Boolean(false)),
            Some(token::Literal::Nil) => Ok(lox_type::LoxType::Nil),
            _ => self.add_diagnostic(expr, "Unhandled literal".to_string()),
        }
    }

    fn evaluate_expr_logical(
        &mut self,
        environment: &mut environment::Environment,
        left: &expr::Expr,
        operator: &token::Token,
        right: &expr::Expr,
    ) -> Result<lox_type::LoxType, unwind::Unwind> {
        let left = self.evaluate_expr(environment, left)?;
        if operator.token_type == token::TokenType::Or {
            if is_truthy(&left) {
                return Ok(left);
            }
        } else if !is_truthy(&left) {
            return Ok(left);
        }

        self.evaluate_expr(environment, right)
    }

    fn evaluate_expr_unary(
        &mut self,
        environment: &mut environment::Environment,
        expression: &expr::Expr,
        operator: &token::Token,
        right: &expr::Expr,
    ) -> Result<lox_type::LoxType, unwind::Unwind> {
        let right = self.evaluate_expr(environment, right)?;
        match operator.token_type {
            token::TokenType::Minus => {
                let right = self.check_number_operand(expression, &right)?;
                Ok(lox_type::LoxType::Number(-1.0 * right))
            }
            token::TokenType::Bang => Ok(lox_type::LoxType::Boolean(!is_truthy(&right))),
            _ => self.add_diagnostic(expression, "Unsupported operand".to_string()),
        }
    }

    fn evaluate_expr_var(
        &self,
        environment: &mut environment::Environment,
        expression: &expr::Expr,
        name: &token::Token,
    ) -> Result<lox_type::LoxType, unwind::Unwind> {
        match environment.get(name) {
            Ok(value) => Ok(value),
            Err(unwind::Unwind::WithError(message)) => self.add_diagnostic(expression, message),
            _ => unreachable!(),
        }
    }

    fn add_diagnostic(
        &self,
        expr: &expr::Expr,
        message: String,
    ) -> Result<lox_type::LoxType, unwind::Unwind> {
        self.reporter
            .add_diagnostic(get_start_location(expr), get_end_location(expr), &message);
        Err(unwind::Unwind::WithError(message))
    }

    fn check_number_operand(
        &self,
        expression: &expr::Expr,
        lox_type: &lox_type::LoxType,
    ) -> Result<f64, unwind::Unwind> {
        match lox_type {
            lox_type::LoxType::Number(value) => Ok(*value),
            _ => Err(self
                .add_diagnostic(expression, "Operand should be a number".to_string())
                .unwrap_err()),
        }
    }

    fn check_string_operand<'a>(
        &self,
        expression: &'a expr::Expr,
        lox_type: &'a lox_type::LoxType,
    ) -> Result<&'a str, unwind::Unwind> {
        match &lox_type {
            lox_type::LoxType::String(string) => Ok(string),
            _ => Err(self
                .add_diagnostic(expression, "Operand should be a string".to_string())
                .unwrap_err()),
        }
    }
}

fn is_truthy(lox_type: &lox_type::LoxType) -> bool {
    match lox_type {
        lox_type::LoxType::Nil => false,
        lox_type::LoxType::Boolean(bool) => *bool,
        _ => true,
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

fn get_start_location(expr: &expr::Expr) -> &location::FileLocation {
    match expr {
        expr::Expr::Assign { name, value: _ } => &name.start,
        expr::Expr::Binary {
            left,
            operator: _,
            right: _,
        } => get_start_location(left),
        expr::Expr::Call {
            callee,
            paren: _,
            arguments: _,
        } => get_start_location(callee),
        expr::Expr::Grouping { expression } => get_start_location(expression),
        expr::Expr::Literal { value } => &value.start,
        expr::Expr::Logical {
            left,
            operator: _,
            right: _,
        } => get_start_location(left),
        expr::Expr::Unary { operator, right: _ } => &operator.start,
        expr::Expr::Variable { name } => &name.start,
    }
}

fn get_end_location(expr: &expr::Expr) -> &location::FileLocation {
    match expr {
        expr::Expr::Assign { name: _, value } => get_end_location(value),
        expr::Expr::Binary {
            left: _,
            operator: _,
            right,
        } => get_end_location(right),
        expr::Expr::Call {
            callee: _,
            paren,
            arguments: _,
        } => &paren.end,
        expr::Expr::Grouping { expression } => get_end_location(expression),
        expr::Expr::Literal { value } => &value.end,
        expr::Expr::Logical {
            left: _,
            operator: _,
            right,
        } => get_end_location(right),
        expr::Expr::Unary { operator: _, right } => get_end_location(right),
        expr::Expr::Variable { name } => &name.end,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        ast_printer, expr, location::FileLocation, parser, reporter::test::TestReporter, scanner,
        token,
    };

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
                Some(token::Literal::String("".to_string())),
            ),
        };

        let tests = vec![
            (
                lox_type::LoxType::String("".to_string()),
                Ok::<&str, unwind::Unwind>(""),
            ),
            (
                lox_type::LoxType::Number(0f64),
                Err::<&str, unwind::Unwind>(unwind::Unwind::WithError(
                    "Operand should be a string".to_string(),
                )),
            ),
            (
                lox_type::LoxType::Nil,
                Err::<&str, unwind::Unwind>(unwind::Unwind::WithError(
                    "Operand should be a string".to_string(),
                )),
            ),
            (
                lox_type::LoxType::Boolean(true),
                Err::<&str, unwind::Unwind>(unwind::Unwind::WithError(
                    "Operand should be a string".to_string(),
                )),
            ),
        ];

        let interpreter = Interpreter::build(&reporter);

        for (value, expected_result) in &tests {
            reporter.reset();
            assert_eq!(
                interpreter.check_string_operand(&expression, value),
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
                Some(token::Literal::String("".to_string())),
            ),
        };

        let tests = vec![
            (
                lox_type::LoxType::String("".to_string()),
                Err::<f64, unwind::Unwind>(unwind::Unwind::WithError(
                    "Operand should be a number".to_string(),
                )),
            ),
            (
                lox_type::LoxType::Number(0f64),
                Ok::<f64, unwind::Unwind>(0f64),
            ),
            (
                lox_type::LoxType::Nil,
                Err::<f64, unwind::Unwind>(unwind::Unwind::WithError(
                    "Operand should be a number".to_string(),
                )),
            ),
            (
                lox_type::LoxType::Boolean(true),
                Err::<f64, unwind::Unwind>(unwind::Unwind::WithError(
                    "Operand should be a number".to_string(),
                )),
            ),
        ];

        let interpreter = Interpreter::build(&reporter);
        for (value, expected_result) in &tests {
            reporter.reset();
            assert_eq!(
                interpreter.check_number_operand(&expression, value),
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
            (lox_type::LoxType::Nil, lox_type::LoxType::Nil, true),
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

    fn test_expressions(tests: Vec<(&str, Result<lox_type::LoxType, unwind::Unwind>)>) {
        let reporter = TestReporter::build();
        for (src, expected_result) in tests {
            let mut interpreter = Interpreter::build(&reporter);
            let mut environment = environment::Environment::new();
            reporter.reset();
            let tokens = scanner::scan_tokens(&reporter, src);
            if let Some(statement) = parser::parse(&reporter, tokens).front() {
                if let stmt::Stmt::Expression { expression } = statement {
                    assert_eq!(
                        interpreter.evaluate_expr(&mut environment, expression),
                        expected_result,
                        "unexpected result: {} != {:?}",
                        ast_printer::print_expr(expression),
                        expected_result
                    );
                } else {
                    panic!("Invalid statement type for '{}'", src);
                };
            } else {
                reporter.print_contents();
                panic!("Statement not found for '{}'", src);
            }
        }
    }

    #[test]
    fn test_plus() {
        let tests = vec![
            (
                "10 + 10 ; ",
                Ok::<lox_type::LoxType, unwind::Unwind>(lox_type::LoxType::Number(20f64)),
            ),
            (
                "\"hello,\" + \" world\";",
                Ok::<lox_type::LoxType, unwind::Unwind>(lox_type::LoxType::String(
                    "hello, world".to_string(),
                )),
            ),
            (
                "10 + \", world\";",
                Err::<lox_type::LoxType, unwind::Unwind>(unwind::Unwind::WithError(
                    "Operands must be two numbers or two strings".to_string(),
                )),
            ),
        ];

        test_expressions(tests);
    }

    #[test]
    fn test_equality() {
        let tests = vec![
            (
                "10 == 10;",
                Ok::<lox_type::LoxType, unwind::Unwind>(lox_type::LoxType::Boolean(true)),
            ),
            (
                "10 != 10;",
                Ok::<lox_type::LoxType, unwind::Unwind>(lox_type::LoxType::Boolean(false)),
            ),
        ];

        test_expressions(tests);
    }

    #[test]
    fn test_binary() {
        let tests = vec![
            (
                "10 - 5;",
                Ok::<lox_type::LoxType, unwind::Unwind>(lox_type::LoxType::Number(5f64)),
            ),
            (
                "10 * 10;",
                Ok::<lox_type::LoxType, unwind::Unwind>(lox_type::LoxType::Number(100f64)),
            ),
        ];

        test_expressions(tests);
    }

    #[test]
    fn test_unary() {
        let tests = vec![
            (
                "-5;",
                Ok::<lox_type::LoxType, unwind::Unwind>(lox_type::LoxType::Number(-5f64)),
            ),
            (
                "!true;",
                Ok::<lox_type::LoxType, unwind::Unwind>(lox_type::LoxType::Boolean(false)),
            ),
        ];

        test_expressions(tests);
    }

    #[test]
    fn test_literal() {
        let tests = vec![
            (
                "\"hello, world\";",
                Ok::<lox_type::LoxType, unwind::Unwind>(lox_type::LoxType::String(
                    "hello, world".to_string(),
                )),
            ),
            (
                "true;",
                Ok::<lox_type::LoxType, unwind::Unwind>(lox_type::LoxType::Boolean(true)),
            ),
        ];

        test_expressions(tests);
    }

    #[test]
    fn test_stmt_var() {
        let tests = vec![
            (
                "var a = \"value\";",
                "a",
                lox_type::LoxType::String("value".to_string()),
            ),
            ("var b;", "b", lox_type::LoxType::Nil),
        ];

        let blank_location = location::FileLocation::new(0, 0);
        let reporter = TestReporter::build();
        for (src, key, expected_value) in tests {
            let mut interpreter = Interpreter::build(&reporter);
            let mut environment = environment::Environment::new();
            reporter.reset();
            let tokens = scanner::scan_tokens(&reporter, src);
            if let Some(statement) = parser::parse(&reporter, tokens).front() {
                if let stmt::Stmt::Var {
                    name: _,
                    initialiser: _,
                } = statement
                {
                    if let Err(unwind::Unwind::WithError(msg)) =
                        interpreter.evaluate_stmt(&mut environment, statement)
                    {
                        panic!("Unexpected error for '{}': {}", src, msg);
                    }
                    let key = token::Token::new(
                        token::TokenType::Identifier,
                        key,
                        blank_location,
                        blank_location,
                        None,
                    );
                    match environment.get(&key) {
                        Ok(value) => {
                            assert_eq!(value, expected_value, "Unexpected value for '{}'", src)
                        }
                        Err(unwind::Unwind::WithError(message)) => {
                            panic!("Unexpected error for '{}': {}", src, message)
                        }
                        _ => unreachable!(),
                    }
                } else {
                    panic!("Invalid statement type for '{}'", src);
                };
            } else {
                reporter.print_contents();
                panic!("Statement not found for '{}'", src);
            }
        }
    }

    #[test]
    fn test_stmt() {
        let tests = vec![
            ("print \"value\";", "[print] \"value\""),
            ("10 + 10;", "[interpreter] 20"),
            ("{true == false;} ", "[interpreter] false"),
            (
                "if (true) print \"then branch\"; ",
                "[print] \"then branch\"",
            ),
            (
                "if (false) print \"then branch\"; else print \"else branch\";",
                "[print] \"else branch\"",
            ),
            ("print \"hi\" or 2 ; ", "[print] \"hi\""),
            ("print nil or \"yes\" ; ", "[print] \"yes\""),
        ];

        let reporter = TestReporter::build();
        for (src, expected_message) in tests {
            let mut interpreter = Interpreter::build(&reporter);
            let mut environment = environment::Environment::new();
            reporter.reset();
            let tokens = scanner::scan_tokens(&reporter, src);
            if let Some(statement) = parser::parse(&reporter, tokens).front() {
                if let Err(unwind::Unwind::WithError(message)) =
                    interpreter.evaluate_stmt(&mut environment, statement)
                {
                    panic!("Unexpected error for '{}': {}", src, message);
                }
                if !reporter.has_message(expected_message) {
                    reporter.print_contents();
                    panic!(
                        "Missing expected message for '{}' expected '{}'",
                        src, expected_message
                    );
                }
            } else {
                reporter.print_contents();
                panic!("Statement not found for '{}'", src);
            }
        }
    }

    #[test]
    fn test_stmts() {
        let tests = vec![
            (
                "var a = 3; if (a > 1) print \"> 1\"; else print \"<= 1\" ;",
                "[print] \"> 1\"",
            ),
            (
                "var a = \"init\"; if (true) a = \"updated\" ; print a;",
                "[print] \"updated\"",
            ),
            (
                "var a = \"init\"; if (false) a = \"updated\" ; print a;",
                "[print] \"init\"",
            ),
            (
                "var a = 1; while ( a < 5) a = a + 1 ; print a;",
                "[print] 5",
            ),
            (
                "fun sayHi(first, last) { print \"Hi, \" + first + \" \" + last; } sayHi(\"Dear\", \"Reader\");",
                "[print] \"Hi, Dear Reader\"",
            ),
            (
                "fun count(n) {if (n> 1) count(n-1); print n; } count(3) ;",
                "[print] 3",
            ),
        ];

        let reporter = TestReporter::build();
        for (src, expected_message) in tests {
            reporter.reset();
            let tokens = scanner::scan_tokens(&reporter, src);
            let statements = parser::parse(&reporter, tokens);
            interpret(&reporter, statements);

            if !reporter.has_message(expected_message) {
                reporter.print_contents();
                panic!(
                    "Missing expected message for '{}' expected '{}'",
                    src, expected_message
                );
            }
        }
    }
}
