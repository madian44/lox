use crate::expr;
use crate::token;

pub fn print(expr: &expr::Expr) -> String {
    match expr {
        expr::Expr::Binary(left, operator, right) => print_binary(left, operator, right),
        expr::Expr::Grouping(expression) => print_grouping(expression),
        expr::Expr::Literal(value) => print_literal(value),
        expr::Expr::Unary(operator, right) => print_unary(operator, right),
    }
}

fn print_binary(left: &expr::Expr, operator: &token::Token, right: &expr::Expr) -> String {
    parenthesize(operator.lexeme, vec![left, right])
}

fn print_grouping(expression: &expr::Expr) -> String {
    parenthesize("group", vec![expression])
}

fn print_literal(value: &token::Literal) -> String {
    let value = match value {
        token::Literal::Number(n) => n.to_string(),
        token::Literal::String(s) => s.to_string(),
        token::Literal::None => "nil".to_string(),
    };
    parenthesize(&value, vec![])
}

fn print_unary(operator: &token::Token, right: &expr::Expr) -> String {
    parenthesize(operator.lexeme, vec![right])
}

fn parenthesize(name: &str, exprs: Vec<&expr::Expr>) -> String {
    let mut output = String::from("(");
    output.push_str(name);
    for expr in exprs {
        output.push(' ');
        output.push_str(&print(expr))
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

        let expression = expr::Expr::binary(
            expr::Expr::unary(
                token::Token::new(
                    token::TokenType::Minus,
                    "-",
                    blank_location.clone(),
                    blank_location.clone(),
                    token::Literal::None,
                ),
                expr::Expr::literal(token::Literal::Number(123.0)),
            ),
            token::Token::new(
                token::TokenType::Star,
                "*",
                blank_location.clone(),
                blank_location.clone(),
                token::Literal::None,
            ),
            expr::Expr::grouping(expr::Expr::literal(token::Literal::Number(45.67))),
        );

        let result = print(&expression);

        assert_eq!("(* (- (123)) (group (45.67)))", result);
    }
}