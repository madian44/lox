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

    fn print_stmt_block(indent: usize, statements: &LinkedList<stmt::Stmt>) -> String {
        let mut result = format!("{}(block\n", indent_string(indent));

        for statement in statements {
            result.push_str(&print_stmt(indent + 1, statement));
        }

        result.push_str(&format!("{})\n", indent_string(indent)));
        result
    }

    fn print_stmt_expr(indent: usize, expr: &expr::Expr) -> String {
        format!(
            "{}{}\n",
            indent_string(indent),
            parenthesize(";", vec![expr])
        )
    }

    fn print_stmt_if(
        indent: usize,
        condition: &expr::Expr,
        then_branch: &stmt::Stmt,
        else_branch: &Option<stmt::Stmt>,
    ) -> String {
        let mut result = format!(
            "{}(if{} ",
            indent_string(indent),
            if else_branch.is_some() { "-else" } else { "" }
        );

        result.push_str(&format!("{}\n", print_expr(condition),));
        result.push_str(&print_stmt(indent + 1, then_branch));
        if let Some(else_branch) = else_branch {
            result.push_str(&print_stmt(indent + 1, else_branch));
        }
        result.push_str(&format!("{})\n", indent_string(indent)));

        result
    }

    fn print_stmt_print(indent: usize, expr: &expr::Expr) -> String {
        format!(
            "{}{}\n",
            indent_string(indent),
            parenthesize("print", vec![expr])
        )
    }

    fn print_stmt_variable(
        indent: usize,
        name: &token::Token,
        initialiser: &Option<expr::Expr>,
    ) -> String {
        let initialiser = match initialiser {
            Some(expr) => format!(" = {}", print_expr(expr)),
            None => "".to_string(),
        };
        format!(
            "{}(var {}{})\n",
            indent_string(indent),
            name.lexeme,
            initialiser
        )
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
            expr::Expr::Assign { name, value } => print_expr_assign(name, value),
            expr::Expr::Binary {
                left,
                operator,
                right,
            } => print_expr_binary(left, operator, right),
            expr::Expr::Call {
                callee,
                paren: _,
                arguments,
            } => print_expr_call(callee, arguments),
            expr::Expr::Grouping { expression } => print_expr_grouping(expression),
            expr::Expr::Literal { value } => print_expr_literal(value),
            expr::Expr::Logical {
                left,
                operator,
                right,
            } => print_expr_logical(left, operator, right),
            expr::Expr::Unary { operator, right } => print_expr_unary(operator, right),
            expr::Expr::Variable { name } => print_expr_variable(name),
        }
    }

    fn print_expr_assign(name: &token::Token, value: &expr::Expr) -> String {
        format!("(= {} {})", name.lexeme, print_expr(value))
    }

    fn print_expr_binary(left: &expr::Expr, operator: &token::Token, right: &expr::Expr) -> String {
        parenthesize(&operator.lexeme, vec![left, right])
    }

    fn print_expr_call(callee: &expr::Expr, arguments: &[expr::Expr]) -> String {
        format!(
            "(call {}{}{})",
            print_expr(callee),
            if !arguments.is_empty() { " " } else { "" },
            arguments
                .iter()
                .map(print_expr)
                .collect::<Vec<String>>()
                .as_slice()
                .join(" ")
        )
    }

    fn print_expr_grouping(expression: &expr::Expr) -> String {
        parenthesize("group", vec![expression])
    }

    fn print_expr_literal(value: &token::Token) -> String {
        match &value.literal {
            Some(token::Literal::Number(n)) => n.to_string(),
            Some(token::Literal::String(s)) => format!("\"{}\"", s),
            Some(token::Literal::Nil) => "Nil".to_string(),
            Some(token::Literal::False) => "false".to_string(),
            Some(token::Literal::True) => "true".to_string(),
            None => "None".to_string(),
        }
    }

    fn print_expr_logical(
        left: &expr::Expr,
        operator: &token::Token,
        right: &expr::Expr,
    ) -> String {
        parenthesize(&operator.lexeme, vec![left, right])
    }

    fn print_expr_unary(operator: &token::Token, right: &expr::Expr) -> String {
        parenthesize(&operator.lexeme, vec![right])
    }

    fn print_expr_variable(name: &token::Token) -> String {
        name.lexeme.clone()
    }

    fn parenthesize(name: &str, exprs: Vec<&expr::Expr>) -> String {
        format!(
            "({}{}{})",
            name,
            if exprs.is_empty() { "" } else { " " },
            exprs
                .iter()
                .map(|a| print_expr(a))
                .collect::<Vec<String>>()
                .as_slice()
                .join(" ")
        )
    }

    fn indent_string(indent: usize) -> String {
        format!("{:width$}", "", width = 4 * indent)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{parser, reporter::test::TestReporter, scanner, Reporter};
    use std::collections::LinkedList;

    fn unindent_string(source: &str) -> String {
        let re = regex::Regex::new(r"\n\s+[|]").unwrap();
        re.replace_all(source, "\n").to_string()
    }

    fn parse_lox(source: &str) -> LinkedList<stmt::Stmt> {
        let reporter = TestReporter::build();

        let tokens = scanner::scan_tokens(&reporter, source);
        if reporter.has_diagnostics() {
            reporter.print_contents();
            panic!("Unexpected errors scanning {}", source);
        }
        let statements = parser::parse(&reporter, tokens);
        if reporter.has_diagnostics() {
            reporter.print_contents();
            panic!("Unexpected errors parsing {}", source);
        }
        statements
    }

    #[test]
    fn tests() {
        let tests = vec![
            (
                "print -123 * (45.67);",
                "(print (* (- 123) (group 45.67)))\n",
            ),
            ("true == true;", "(; (== true true))\n"),
            ("var a = true;", "(var a = true)\n"),
            ("var b;", "(var b)\n"),
            (
                "{ var a = Nil ; a = false; }",
                "(block
                |    (var a = Nil)
                |    (; (= a false))
                |)\n",
            ),
            (
                "if ( true or \"hello, world\" ) print \"then branch\" ; else print \"else branch\" ;",
                "(if-else (or true \"hello, world\")
                |    (print \"then branch\")
                |    (print \"else branch\")
                |)\n",
            ),
            (
                "while (true) print \"body\" ;",
                "(while true
                |    (print \"body\")
                |)\n"
            ),
            (
                "callee();",
                "(; (call callee))\n"
            ),
            ("callee(a, b);", "(; (call callee a b))\n"),
        ];

        for (source, expected_output) in tests {
            let expected_output = unindent_string(expected_output);
            let statements = parse_lox(source);
            let mut output = String::new();
            for statement in statements {
                output.push_str(&print_stmt(&statement));
            }
            assert_eq!(
                output, expected_output,
                "Unexpected output for '{}'",
                source
            );
        }
    }
}
