#[cfg(test)]
use crate::expr;
use crate::stmt;

pub fn print_stmt(stmt: &stmt::Stmt) -> String {
    internal::print_stmt(0, stmt)
}

#[cfg(test)]
pub fn print_expr(expr: &expr::Expr) -> String {
    internal::print_expr(expr)
}

mod internal {
    use crate::{expr, stmt, token};
    use std::collections::LinkedList;

    pub fn print_stmt(indent: usize, stmt: &stmt::Stmt) -> String {
        match stmt {
            stmt::Stmt::Block { statements } => print_stmt_block(indent, statements),
            stmt::Stmt::Expression { expression } => print_stmt_expr(indent, expression),
            stmt::Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => print_stmt_if(indent, condition, then_branch, else_branch),
            stmt::Stmt::Print { value } => print_stmt_print(indent, value),
            stmt::Stmt::Var { name, initialiser } => print_stmt_variable(indent, name, initialiser),
            stmt::Stmt::While { condition, body } => print_stmt_while(indent, condition, body),
        }
    }

    fn print_stmt_expr(indent: usize, expr: &expr::Expr) -> String {
        format!(
            "{}(expression {})\n",
            indent_string(indent),
            print_expr(expr)
        )
    }

    fn print_stmt_print(indent: usize, expr: &expr::Expr) -> String {
        format!("{}(print {})\n", indent_string(indent), print_expr(expr))
    }

    fn print_stmt_variable(
        indent: usize,
        name: &token::Token,
        initialiser: &Option<expr::Expr>,
    ) -> String {
        let initialiser = match initialiser {
            Some(expr) => format!(" {}", print_expr(expr)),
            None => "".to_string(),
        };
        format!(
            "{}(var ({}){})\n",
            indent_string(indent),
            name.lexeme,
            initialiser
        )
    }

    fn print_stmt_block(indent: usize, statements: &LinkedList<stmt::Stmt>) -> String {
        let mut result = format!("{}(block\n", indent_string(indent));

        for statement in statements {
            result.push_str(&print_stmt(indent + 1, statement));
        }

        result.push_str(&format!("{})\n", indent_string(indent)));
        result
    }

    fn print_stmt_if(
        indent: usize,
        condition: &expr::Expr,
        then_branch: &stmt::Stmt,
        else_branch: &Option<stmt::Stmt>,
    ) -> String {
        let mut result = format!("{}(if ", indent_string(indent));

        result.push_str(&format!("{}\n", print_expr(condition),));
        result.push_str(&print_stmt(indent + 1, then_branch));
        if let Some(else_branch) = else_branch {
            result.push_str(&print_stmt(indent + 1, else_branch));
        }
        result.push_str(&format!("{})\n", indent_string(indent)));

        result
    }

    fn print_stmt_while(indent: usize, condition: &expr::Expr, body: &stmt::Stmt) -> String {
        let mut result = format!(
            "{}(while {}\n",
            indent_string(indent),
            print_expr(condition),
        );
        result.push_str(&print_stmt(indent + 1, body));
        result.push_str(&format!("{})\n", indent_string(indent)));
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
        format!("(= ({}) {})", name.lexeme, print_expr(value))
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
            Some(token::Literal::String(s)) => format!("\"{}\"", s),
            Some(token::Literal::Nil) => "Nil".to_string(),
            Some(token::Literal::False) => "False".to_string(),
            Some(token::Literal::True) => "True".to_string(),
            None => "None".to_string(),
        };
        parenthesize(&value, vec![])
    }

    fn print_expr_logical(
        left: &expr::Expr,
        operator: &token::Token,
        right: &expr::Expr,
    ) -> String {
        format!(
            "({1} {0} {2})",
            print_expr(left),
            operator.lexeme,
            print_expr(right)
        )
    }

    fn print_expr_unary(operator: &token::Token, right: &expr::Expr) -> String {
        parenthesize(&operator.lexeme, vec![right])
    }

    fn print_expr_variable(name: &token::Token) -> String {
        format!("({})", name.lexeme.clone())
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

    fn indent_string(indent: usize) -> String {
        format!("{:width$}", "", width = 4 * indent)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{location, token};
    use std::collections::LinkedList;

    fn unindent_string(source: &str) -> String {
        let re = regex::Regex::new(r"\n\s+[|]").unwrap();
        re.replace_all(source, "\n").to_string()
    }

    #[test]
    fn print_statement() {
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

        assert_eq!("(print (* (- (123)) (group (45.67))))\n", result);
    }

    #[test]
    fn expression_statement() {
        let blank_location = location::FileLocation::new(0, 0);

        let expression = expr::Expr::build_logical(
            expr::Expr::build_literal(token::Token::new(
                token::TokenType::True,
                "true",
                blank_location,
                blank_location,
                Some(token::Literal::True),
            )),
            token::Token::new(
                token::TokenType::EqualEqual,
                "==",
                blank_location,
                blank_location,
                None,
            ),
            expr::Expr::build_literal(token::Token::new(
                token::TokenType::True,
                "true",
                blank_location,
                blank_location,
                Some(token::Literal::True),
            )),
        );

        let statement = stmt::Stmt::Expression { expression };

        let result = print_stmt(&statement);

        assert_eq!("(expression (== (True) (True)))\n", result);
    }

    #[test]
    fn var_statement() {
        let blank_location = location::FileLocation::new(0, 0);

        let initialiser = expr::Expr::build_literal(token::Token::new(
            token::TokenType::True,
            "true",
            blank_location,
            blank_location,
            Some(token::Literal::True),
        ));

        let name = token::Token::new(
            token::TokenType::Identifier,
            "a",
            blank_location,
            blank_location,
            None,
        );

        let statement = stmt::Stmt::Var {
            name,
            initialiser: Some(initialiser),
        };

        let result = print_stmt(&statement);

        assert_eq!("(var (a) (True))\n", result);

        let name = token::Token::new(
            token::TokenType::Identifier,
            "b",
            blank_location,
            blank_location,
            None,
        );

        let statement = stmt::Stmt::Var {
            name,
            initialiser: None,
        };

        let result = print_stmt(&statement);

        assert_eq!("(var (b))\n", result);
    }

    #[test]
    fn block_statement() {
        let blank_location = location::FileLocation::new(0, 0);

        let initialiser = expr::Expr::build_literal(token::Token::new(
            token::TokenType::True,
            "true",
            blank_location,
            blank_location,
            Some(token::Literal::True),
        ));

        let name = token::Token::new(
            token::TokenType::Identifier,
            "a",
            blank_location,
            blank_location,
            None,
        );

        let mut statements = LinkedList::new();
        statements.push_back(stmt::Stmt::Var {
            name,
            initialiser: Some(initialiser),
        });

        let name = token::Token::new(
            token::TokenType::Identifier,
            "a",
            blank_location,
            blank_location,
            None,
        );

        let value = expr::Expr::build_literal(token::Token::new(
            token::TokenType::False,
            "false",
            blank_location,
            blank_location,
            Some(token::Literal::False),
        ));

        let assign = expr::Expr::build_assign(name, value);

        statements.push_back(stmt::Stmt::Expression { expression: assign });

        let statement = stmt::Stmt::Block { statements };

        let result = print_stmt(&statement);

        assert_eq!(
            unindent_string(
                "(block
                    |    (var (a) (True))
                    |    (expression (= (a) (False)))
                    |)\n"
            ),
            result
        );
    }

    #[test]
    fn if_statement() {
        let blank_location = location::FileLocation::new(0, 0);

        let left = expr::Expr::build_literal(token::Token::new(
            token::TokenType::True,
            "true",
            blank_location,
            blank_location,
            Some(token::Literal::True),
        ));

        let operator = token::Token::new(
            token::TokenType::Or,
            "or",
            blank_location,
            blank_location,
            None,
        );

        let right = expr::Expr::build_literal(token::Token::new(
            token::TokenType::String,
            "\"hello, world\"",
            blank_location,
            blank_location,
            Some(token::Literal::String("hello, world".to_string())),
        ));

        let then_branch = expr::Expr::build_literal(token::Token::new(
            token::TokenType::String,
            "\"then branch\"",
            blank_location,
            blank_location,
            Some(token::Literal::String("then branch".to_string())),
        ));

        let then_branch = stmt::Stmt::Print { value: then_branch };

        let else_branch = expr::Expr::build_literal(token::Token::new(
            token::TokenType::String,
            "\"else branch\"",
            blank_location,
            blank_location,
            Some(token::Literal::String("else branch".to_string())),
        ));

        let else_branch = stmt::Stmt::Print { value: else_branch };

        let condition = expr::Expr::build_binary(left, operator, right);

        let statement = stmt::Stmt::If {
            condition,
            then_branch: Box::new(then_branch),
            else_branch: Box::new(Some(else_branch)),
        };

        let result = print_stmt(&statement);

        assert_eq!(
            unindent_string(
                "(if (or (True) (\"hello, world\"))
                    |    (print (\"then branch\"))
                    |    (print (\"else branch\"))
                    |)\n"
            ),
            result
        );
    }

    #[test]
    fn while_statement() {
        let blank_location = location::FileLocation::new(0, 0);

        let condition = expr::Expr::build_literal(token::Token::new(
            token::TokenType::True,
            "true",
            blank_location,
            blank_location,
            Some(token::Literal::True),
        ));

        let body = expr::Expr::build_literal(token::Token::new(
            token::TokenType::String,
            "\"body\"",
            blank_location,
            blank_location,
            Some(token::Literal::String("body".to_string())),
        ));

        let body = stmt::Stmt::Print { value: body };

        let statement = stmt::Stmt::While {
            condition,
            body: Box::new(body),
        };

        let result = print_stmt(&statement);

        assert_eq!(
            unindent_string(
                "(while (True)
                    |    (print (\"body\"))
                    |)\n"
            ),
            result
        );
    }
}
