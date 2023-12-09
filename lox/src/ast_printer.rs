use crate::{expr, stmt, token};
use std::collections::LinkedList;

pub fn print_stmt(stmt: &stmt::Stmt) -> String {
    match stmt {
        stmt::Stmt::Expression { expression } => format!("{} ;", print_expr(expression)),
        stmt::Stmt::Print { value } => format!("PRINT {} ;", print_expr(value)),
        stmt::Stmt::Var { name, initialiser } => print_stmt_variable(name, initialiser),
        stmt::Stmt::Block { statements } => print_stmt_block(statements),
        stmt::Stmt::If {
            condition,
            then_branch,
            else_branch,
        } => print_stmt_if(condition, then_branch, else_branch),
    }
}

fn print_stmt_variable(name: &token::Token, initialiser: &Option<expr::Expr>) -> String {
    let initialiser = match initialiser {
        Some(expr) => format!("= {} ", print_expr(expr)),
        None => "".to_string(),
    };
    format!("VAR {} {};", name.lexeme, initialiser)
}

fn print_stmt_block(statements: &LinkedList<stmt::Stmt>) -> String {
    let mut result = String::from("{\n");

    for statement in statements {
        result.push_str(&print_stmt(statement));
        result.push('\n');
    }

    result.push('}');
    result
}

fn print_stmt_if(
    condition: &expr::Expr,
    then_statement: &stmt::Stmt,
    else_branch: &Option<stmt::Stmt>,
) -> String {
    let mut result = String::from("IF ");

    result.push_str(&format!(
        "({}) THEN {}",
        print_expr(condition),
        print_stmt(then_statement)
    ));
    if let Some(else_branch) = else_branch {
        result.push_str(&format!(" ELSE {}", print_stmt(else_branch)))
    }

    result
}

pub fn print_expr(expr: &expr::Expr) -> String {
    match expr {
        expr::Expr::Binary {
            left,
            operator,
            right,
        } => print_expr_binary(left, operator, right),
        expr::Expr::Grouping { expression } => print_expr_grouping(expression),
        expr::Expr::Literal { value } => print_expr_literal(value),
        expr::Expr::Logical {
            left,
            operator,
            right,
        } => print_expr_logical(left, operator, right),
        expr::Expr::Unary { operator, right } => print_expr_unary(operator, right),
        expr::Expr::Variable { name } => print_expr_variable(name),
        expr::Expr::Assign { name, value } => print_expr_assign(name, value),
    }
}

fn print_expr_assign(name: &token::Token, value: &expr::Expr) -> String {
    format!("{} = {}", name.lexeme, print_expr(value))
}

fn print_expr_binary(left: &expr::Expr, operator: &token::Token, right: &expr::Expr) -> String {
    parenthesize(&operator.lexeme, vec![left, right])
}

fn print_expr_grouping(expression: &expr::Expr) -> String {
    parenthesize("group", vec![expression])
}

fn print_expr_literal(value: &token::Token) -> String {
    let value = match &value.literal {
        Some(token::Literal::Number(n)) => n.to_string(),
        Some(token::Literal::String(s)) => s.to_string(),
        Some(token::Literal::Nil) => "Nil".to_string(),
        Some(token::Literal::False) => "False".to_string(),
        Some(token::Literal::True) => "True".to_string(),
        None => "None".to_string(),
    };
    parenthesize(&value, vec![])
}

fn print_expr_logical(left: &expr::Expr, operator: &token::Token, right: &expr::Expr) -> String {
    format!(
        "{} {} {}",
        print_expr(left),
        operator.lexeme,
        print_expr(right)
    )
}

fn print_expr_unary(operator: &token::Token, right: &expr::Expr) -> String {
    parenthesize(&operator.lexeme, vec![right])
}

fn print_expr_variable(name: &token::Token) -> String {
    name.lexeme.clone()
}

fn parenthesize(name: &str, exprs: Vec<&expr::Expr>) -> String {
    let mut output = String::from("(");
    output.push_str(name);
    for expr in exprs {
        output.push(' ');
        output.push_str(&print_expr(expr))
    }

    output.push(')');
    output
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::location;

    #[test]
    fn example1() {
        let blank_location = location::FileLocation::new(0, 0);

        let expression = expr::Expr::build_binary(
            expr::Expr::build_unary(
                token::Token::new(
                    token::TokenType::Minus,
                    "-",
                    blank_location,
                    blank_location,
                    None,
                ),
                expr::Expr::build_literal(token::Token::new(
                    token::TokenType::Number,
                    "123.0",
                    blank_location,
                    blank_location,
                    Some(token::Literal::Number(123.0)),
                )),
            ),
            token::Token::new(
                token::TokenType::Star,
                "*",
                blank_location,
                blank_location,
                None,
            ),
            expr::Expr::build_grouping(expr::Expr::build_literal(token::Token::new(
                token::TokenType::Number,
                "45.67",
                blank_location,
                blank_location,
                Some(token::Literal::Number(45.67)),
            ))),
        );

        let statement = stmt::Stmt::Print { value: expression };

        let result = print_stmt(&statement);

        assert_eq!("PRINT (* (- (123)) (group (45.67))) ;", result);
    }
}
