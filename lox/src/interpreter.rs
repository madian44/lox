mod class;
mod environment;
mod function;
mod instance;
mod lox_type;
mod native_functions;
mod unwind;

use crate::interpreter::lox_type::Callable;
use crate::{expr, location, reporter, stmt, token};
use std::collections::{HashMap, LinkedList};

pub fn interpret(
    reporter: &dyn reporter::Reporter,
    depths: &HashMap<usize, usize>,
    statements: LinkedList<stmt::Stmt>,
) {
    let mut environment = environment::Environment::new();
    Interpreter::define_native_functions(&mut environment);

    let _ = interpret_with_environment(reporter, depths, &mut environment, &statements);
}

fn interpret_with_environment(
    reporter: &dyn reporter::Reporter,
    depths: &HashMap<usize, usize>,
    environment: &mut environment::Environment,
    statements: &LinkedList<stmt::Stmt>,
) -> Result<(), unwind::Unwind> {
    let interpreter = Interpreter::new(reporter, depths, statements);

    interpreter.interpret_statements(environment)
}

struct Interpreter<'r> {
    reporter: &'r dyn reporter::Reporter,
    depths: &'r HashMap<usize, usize>,
    statements: &'r LinkedList<stmt::Stmt>,
}

impl<'r> Interpreter<'r> {
    fn new(
        reporter: &'r dyn reporter::Reporter,
        depths: &'r HashMap<usize, usize>,
        statements: &'r LinkedList<stmt::Stmt>,
    ) -> Self {
        Self {
            reporter,
            depths,
            statements,
        }
    }

    fn define_native_functions(environment: &mut environment::Environment) {
        environment.define("clock", native_functions::clock());
    }

    fn interpret_statements(
        &self,
        environment: &mut environment::Environment,
    ) -> Result<(), unwind::Unwind> {
        for statement in self.statements {
            match self.evaluate_stmt(environment, statement) {
                Err(unwind::Unwind::WithError(message)) => {
                    self.reporter.add_message(&message);
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

    fn evaluate_stmt(
        &self,
        environment: &mut environment::Environment,
        statement: &stmt::Stmt,
    ) -> Result<(), unwind::Unwind> {
        match statement {
            stmt::Stmt::Block { statements } => self.evaluate_stmt_block(environment, statements),
            stmt::Stmt::Class { name, methods } => {
                self.evalute_stmt_class(environment, name, methods)
            }
            stmt::Stmt::Expression { expression } => {
                self.evaluate_stmt_expression(environment, expression)
            }
            stmt::Stmt::Function { function } => self.evaluate_stmt_function(environment, function),
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
        &self,
        environment: &mut environment::Environment,
        statements: &LinkedList<stmt::Stmt>,
    ) -> Result<(), unwind::Unwind> {
        let mut environment = environment::Environment::new_with_enclosing(environment);
        for statement in statements {
            self.evaluate_stmt(&mut environment, statement)?;
        }
        Ok(())
    }

    fn evalute_stmt_class(
        &self,
        environment: &mut environment::Environment,
        name: &token::Token,
        methods: &LinkedList<stmt::Stmt>,
    ) -> Result<(), unwind::Unwind> {
        environment.define(&name.lexeme, lox_type::LoxType::Nil);

        let methods = methods
            .iter()
            .map(|method| {
                if let stmt::Stmt::Function { function } = method {
                    let f = function::Function::new(
                        environment,
                        function.clone(),
                        function.name().lexeme == "init",
                    );
                    (
                        f.name().to_string(),
                        lox_type::LoxType::Function { function: f },
                    )
                } else {
                    panic!("Unexpected statement")
                }
            })
            .collect::<HashMap<String, lox_type::LoxType>>();

        let class = class::Class::new(&name.lexeme, methods);
        let class = lox_type::LoxType::Class { class };
        let _ = environment.assign_at(Some(0), &name.lexeme, class);
        Ok(())
    }

    fn evaluate_stmt_expression(
        &self,
        environment: &mut environment::Environment,
        expr: &expr::Expr,
    ) -> Result<(), unwind::Unwind> {
        match self.evaluate_expr(environment, expr) {
            Ok(_r) => {
                //                self.reporter.add_message(&format!("[interpreter] {r}"));
                Ok(())
            }
            Err(err) => Err(err),
        }
    }

    fn evaluate_stmt_function(
        &self,
        environment: &mut environment::Environment,
        function: &stmt::function::Function,
    ) -> Result<(), unwind::Unwind> {
        let name = function.name().clone();
        let function = function::Function::new(environment, function.clone(), false);
        let function = lox_type::LoxType::Function { function };
        environment.define(&name.lexeme, function);
        Ok(())
    }

    fn evaluate_stmt_if(
        &self,
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
        &self,
        environment: &mut environment::Environment,
        expr: &expr::Expr,
    ) -> Result<(), unwind::Unwind> {
        let result = self.evaluate_expr(environment, expr)?;
        self.reporter.add_message(&format!("[print] {}", result));
        Ok(())
    }

    fn evaluate_stmt_return(
        &self,
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
        &self,
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
        &self,
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
        &self,
        environment: &mut environment::Environment,
        expression: &expr::Expr,
    ) -> Result<lox_type::LoxType, unwind::Unwind> {
        match expression {
            expr::Expr::Assign {
                id, name, value, ..
            } => self.evaluate_expr_assign(environment, expression, id, name, value),
            expr::Expr::Binary {
                left,
                operator,
                right,
                ..
            } => self.evaluate_expr_binary(environment, expression, left, operator, right),
            expr::Expr::Call {
                callee,
                paren,
                arguments,
                ..
            } => self.evaluate_expr_call(environment, callee, paren, arguments),
            expr::Expr::Get { object, name, .. } => {
                self.evaluate_expr_get(environment, object, name)
            }
            expr::Expr::Grouping { expression, .. } => self.evaluate_expr(environment, expression),
            expr::Expr::Literal { value, .. } => self.evaluate_expr_literal(expression, value),
            expr::Expr::Logical {
                left,
                operator,
                right,
                ..
            } => self.evaluate_expr_logical(environment, left, operator, right),
            expr::Expr::Set {
                object,
                name,
                value,
                ..
            } => self.evaluate_expr_set(environment, object, name, value),
            expr::Expr::This { id, keyword, .. } => {
                self.evaluate_expr_this(environment, id, keyword)
            }
            expr::Expr::Unary {
                operator, right, ..
            } => self.evaluate_expr_unary(environment, expression, operator, right),
            expr::Expr::Variable { id, name, .. } => {
                self.evaluate_expr_var(environment, expression, id, name)
            }
        }
    }

    fn evaluate_expr_assign(
        &self,
        environment: &mut environment::Environment,
        expression: &expr::Expr,
        id: &usize,
        name: &token::Token,
        value: &expr::Expr,
    ) -> Result<lox_type::LoxType, unwind::Unwind> {
        let value = self.evaluate_expr(environment, value)?;
        if let Err(unwind::Unwind::WithError(message)) =
            environment.assign_at(self.depths.get(id).cloned(), &name.lexeme, value.clone())
        {
            self.add_diagnostic(expression, message)?;
        }
        Ok(value)
    }

    fn evaluate_expr_binary(
        &self,
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
        &self,
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

        let result = self.call_function(actual_callee, callee, args);
        match result {
            Err(unwind::Unwind::WithError(_)) => result,
            Err(unwind::Unwind::WithResult(_)) => result,
            Ok(value) => Ok(value),
        }
    }

    fn call_function(
        &self,
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
            lox_type::LoxType::Function { function, .. } => {
                check_arity(function.arity())?;
                match function.call(self.reporter, self.depths, arguments) {
                    Err(unwind::Unwind::WithError(message)) => {
                        Err(unwind::Unwind::WithError(message))
                    }
                    Err(unwind::Unwind::WithResult(value)) => Ok(value),
                    _ => Ok(lox_type::LoxType::Nil),
                }
            }
            lox_type::LoxType::NativeFunction { callable, .. } => {
                check_arity(callable.arity())?;
                callable.call(arguments)
            }
            lox_type::LoxType::Class { class } => {
                check_arity(class.arity())?;
                class.call(self.reporter, self.depths, arguments)
            }
            _ => Err(self
                .add_diagnostic(expr, "Can only call functions and classes".to_string())
                .unwrap_err()),
        }
    }

    fn evaluate_expr_get(
        &self,
        environment: &mut environment::Environment,
        expression: &expr::Expr,
        name: &token::Token,
    ) -> Result<lox_type::LoxType, unwind::Unwind> {
        let object = self.evaluate_expr(environment, expression)?;
        match lox_type::LoxType::get_instance_value(&object, &name.lexeme) {
            Ok(value) => Ok(value),
            Err(unwind::Unwind::WithError(message)) => self.add_diagnostic(expression, message),
            _ => unreachable!(),
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
        &self,
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

    fn evaluate_expr_set(
        &self,
        environment: &mut environment::Environment,
        expression: &expr::Expr,
        name: &token::Token,
        value: &expr::Expr,
    ) -> Result<lox_type::LoxType, unwind::Unwind> {
        let instance = self.evaluate_expr(environment, expression)?;
        let value = self.evaluate_expr(environment, value)?;
        if let Err(unwind::Unwind::WithError(message)) =
            lox_type::LoxType::set_instance_value(&instance, &name.lexeme, value.clone())
        {
            self.add_diagnostic(expression, message)?;
        }
        Ok(value)
    }

    fn evaluate_expr_this(
        &self,
        environment: &mut environment::Environment,
        id: &usize,
        keyword: &token::Token,
    ) -> Result<lox_type::LoxType, unwind::Unwind> {
        match self.look_up_variable(environment, id, keyword) {
            Ok(value) => Ok(value),
            Err(unwind::Unwind::WithError(message)) => self.add_diagnostic(keyword, message),
            _ => unreachable!(),
        }
    }

    fn evaluate_expr_unary(
        &self,
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
        id: &usize,
        name: &token::Token,
    ) -> Result<lox_type::LoxType, unwind::Unwind> {
        match self.look_up_variable(environment, id, name) {
            Ok(value) => Ok(value),
            Err(unwind::Unwind::WithError(message)) => self.add_diagnostic(expression, message),
            _ => unreachable!(),
        }
    }

    fn look_up_variable(
        &self,
        environment: &environment::Environment,
        id: &usize,
        name: &token::Token,
    ) -> Result<lox_type::LoxType, unwind::Unwind> {
        let depth = self.depths.get(id).cloned();
        environment.get_at(depth, &name.lexeme)
    }

    fn add_diagnostic(
        &self,
        provider: impl location::ProvideLocation,
        message: String,
    ) -> Result<lox_type::LoxType, unwind::Unwind> {
        self.reporter
            .add_diagnostic(provider.start(), provider.end(), &message);
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        ast_printer, expr, location::FileLocation, parser, reporter::test::TestReporter, resolver,
        scanner, token,
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
        let reporter = TestReporter::new();
        let blank_location = FileLocation::new(0, 0);
        let expression = expr::Expr::Literal {
            id: 0,
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

        let depths = HashMap::<usize, usize>::new();
        let statements = LinkedList::<stmt::Stmt>::new();
        let interpreter = Interpreter::new(&reporter, &depths, &statements);

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
        let reporter = TestReporter::new();
        let blank_location = FileLocation::new(0, 0);
        let expression = expr::Expr::Literal {
            id: 0,
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

        let depths = HashMap::<usize, usize>::new();
        let statements = LinkedList::<stmt::Stmt>::new();
        let interpreter = Interpreter::new(&reporter, &depths, &statements);
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
        let reporter = TestReporter::new();
        let depths = HashMap::<usize, usize>::new();
        let statements = LinkedList::<stmt::Stmt>::new();
        for (src, expected_result) in tests {
            let interpreter = Interpreter::new(&reporter, &depths, &statements);
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
    fn test_logical() {
        let tests = vec![
            (
                "true and true;",
                Ok::<lox_type::LoxType, unwind::Unwind>(lox_type::LoxType::Boolean(true)),
            ),
            (
                "true and false;",
                Ok::<lox_type::LoxType, unwind::Unwind>(lox_type::LoxType::Boolean(false)),
            ),
            (
                "false and true;",
                Ok::<lox_type::LoxType, unwind::Unwind>(lox_type::LoxType::Boolean(false)),
            ),
            (
                "false and false;",
                Ok::<lox_type::LoxType, unwind::Unwind>(lox_type::LoxType::Boolean(false)),
            ),
            (
                "true or true;",
                Ok::<lox_type::LoxType, unwind::Unwind>(lox_type::LoxType::Boolean(true)),
            ),
            (
                "true or false;",
                Ok::<lox_type::LoxType, unwind::Unwind>(lox_type::LoxType::Boolean(true)),
            ),
            (
                "false or true;",
                Ok::<lox_type::LoxType, unwind::Unwind>(lox_type::LoxType::Boolean(true)),
            ),
            (
                "false or false;",
                Ok::<lox_type::LoxType, unwind::Unwind>(lox_type::LoxType::Boolean(false)),
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

        let reporter = TestReporter::new();
        let depths = HashMap::<usize, usize>::new();
        let statements = LinkedList::<stmt::Stmt>::new();
        for (src, key, expected_value) in tests {
            let interpreter = Interpreter::new(&reporter, &depths, &statements);
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
                    match environment.get_at(None, key) {
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
            ("print 10 + 10;", "[print] 20"),
            ("{print true == false;} ", "[print] false"),
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

        let reporter = TestReporter::new();
        let depths = HashMap::<usize, usize>::new();
        let statements = LinkedList::<stmt::Stmt>::new();
        for (src, expected_message) in tests {
            let interpreter = Interpreter::new(&reporter, &depths, &statements);
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
            (
                "class Bagel{} var b = Bagel() ; b.val = \"hello field\" ; print b.val ;",
                "[print] \"hello field\"",
            ),
            (
                "class Box {} fun notMethod(argument) { print \"called with '\" + argument + \"'\"; } var box = Box() ; box.function = notMethod; box.function(\"hello\");",
                "[print] \"called with 'hello'\"",
            ),
            (
                "class Bacon { eat() { print \"chewy\"; } } Bacon().eat();",
                "[print] \"chewy\"",
            ),
            (
                "fun bacon() { return \"chewy\"; } print bacon();",
                "[print] \"chewy\"",
            ),
            (
                "class Bacon { init() { this.how = \"chewy\"; } } var b = Bacon(); print b.how;",
                "[print] \"chewy\"",
            ),
            (
                "print (10+10)+2;",
                "[print] 22",
            ),
            (
                "class Thing {} var a = Thing(); print (a == a);",
                "[print] false",
            ),
        ];

        let reporter = TestReporter::new();
        for (src, expected_message) in tests {
            reporter.reset();
            let tokens = scanner::scan_tokens(&reporter, src);
            let statements = parser::parse(&reporter, tokens);
            let depths = resolver::resolve(&reporter, &statements);
            interpret(&reporter, &depths, statements);

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
