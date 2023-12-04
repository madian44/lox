use crate::{expr, stmt, token};

pub fn print_stmt(stmt: &stmt::Stmt) -> String {
    match stmt {
        stmt::Stmt::Expression { expression } => format!("{} ;", print_expr(expression)),
        stmt::Stmt::Print { value } => format!("PRINT {} ;", print_expr(value)),
    }
}

pub fn print_expr(expr: &expr::Expr) -> String {
    match expr {
        expr::Expr::Binary {
            left,
            operator,
            right,
        } => print_binary(left, operator, right),
        expr::Expr::Grouping { expression } => print_grouping(expression),
        expr::Expr::Literal { value } => print_literal(value),
        expr::Expr::Unary { operator, right } => print_unary(operator, right),
    }
}

fn print_binary(left: &expr::Expr, operator: &token::Token, right: &expr::Expr) -> String {
    parenthesize(&operator.lexeme, vec![left, right])
}

fn print_grouping(expression: &expr::Expr) -> String {
    parenthesize("group", vec![expression])
}

fn print_literal(value: &token::Token) -> String {
    let value = match &value.literal {
        token::Literal::Number(n) => n.to_string(),
        token::Literal::String(s) => s.to_string(),
        token::Literal::None => "None".to_string(),
        token::Literal::Nil => "Nil".to_string(),
        token::Literal::False => "False".to_string(),
        token::Literal::True => "True".to_string(),
    };
    parenthesize(&value, vec![])
}

fn print_unary(operator: &token::Token, right: &expr::Expr) -> String {
    parenthesize(&operator.lexeme, vec![right])
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
                    token::Literal::None,
                ),
                expr::Expr::build_literal(token::Token::new(
                    token::TokenType::Number,
                    "123.0",
                    blank_location,
                    blank_location,
                    token::Literal::Number(123.0),
                )),
            ),
            token::Token::new(
                token::TokenType::Star,
                "*",
                blank_location,
                blank_location,
                token::Literal::None,
            ),
            expr::Expr::build_grouping(expr::Expr::build_literal(token::Token::new(
                token::TokenType::Number,
                "45.67",
                blank_location,
                blank_location,
                token::Literal::Number(45.67),
            ))),
        );

        let statement = stmt::Stmt::Print { value: expression };

        let result = print_stmt(&statement);

        assert_eq!("PRINT (* (- (123)) (group (45.67))) ;", result);
    }
}
