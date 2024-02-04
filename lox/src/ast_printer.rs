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
            stmt::Stmt::Class {
                name,
                superclass,
                methods,
            } => print_stmt_class(indent, name, superclass, methods),
            stmt::Stmt::Expression { expression } => print_stmt_expr(indent, expression),
            stmt::Stmt::Function { function } => print_stmt_function(indent, function),
            stmt::Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => print_stmt_if(indent, condition, then_branch, else_branch),
            stmt::Stmt::Print { value } => print_stmt_print(indent, value),
            stmt::Stmt::Return { keyword, value } => print_stmt_return(indent, keyword, value),
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

    fn print_stmt_class(
        indent: usize,
        name: &token::Token,
        superclass: &Option<expr::Expr>,
        methods: &LinkedList<stmt::Stmt>,
    ) -> String {
        let mut result = format!(
            "{}(class {}{}\n",
            indent_string(indent),
            name.lexeme,
            superclass
                .as_ref()
                .map_or("".to_string(), |superclass| format!(
                    " < {}",
                    print_expr(superclass)
                ))
        );

        for method in methods {
            result.push_str(&print_stmt(indent + 1, method));
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

    fn print_stmt_function(indent: usize, function: &stmt::function::Function) -> String {
        let mut result = format!(
            "{}(fun {}({})\n",
            indent_string(indent),
            function.name().lexeme,
            function
                .params()
                .iter()
                .map(|p| p.lexeme.clone())
                .collect::<Vec<String>>()
                .as_slice()
                .join(" ")
        );
        for statement in function.body() {
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

    fn print_stmt_return(
        indent: usize,
        _keyword: &token::Token,
        value: &Option<expr::Expr>,
    ) -> String {
        format!(
            "{}{}\n",
            indent_string(indent),
            if let Some(value) = value {
                parenthesize("return", vec![value])
            } else {
                "(return)".to_string()
            }
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
            expr::Expr::Assign { name, value, .. } => print_expr_assign(name, value),
            expr::Expr::Binary {
                left,
                operator,
                right,
                ..
            } => print_expr_binary(left, operator, right),
            expr::Expr::Call {
                callee, arguments, ..
            } => print_expr_call(callee, arguments),
            expr::Expr::Get { object, name, .. } => print_expr_get(object, name),
            expr::Expr::Grouping { expression, .. } => print_expr_grouping(expression),
            expr::Expr::InvalidGet { object, name, .. } => print_expr_get(object, name),
            expr::Expr::InvalidSuper {
                keyword, method, ..
            } => print_expr_super(keyword, method),
            expr::Expr::Literal { value, .. } => print_expr_literal(value),
            expr::Expr::Logical {
                left,
                operator,
                right,
                ..
            } => print_expr_logical(left, operator, right),
            expr::Expr::Set {
                object,
                name,
                value,
                ..
            } => print_expr_set(object, name, value),
            expr::Expr::Super {
                keyword, method, ..
            } => print_expr_super(keyword, method),
            expr::Expr::This { .. } => print_expr_this(),
            expr::Expr::Unary {
                operator, right, ..
            } => print_expr_unary(operator, right),
            expr::Expr::Variable { name, .. } => print_expr_variable(name),
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

    fn print_expr_get(object: &expr::Expr, name: &token::Token) -> String {
        format!("({}.{})", print_expr(object), name.lexeme)
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

    fn print_expr_set(object: &expr::Expr, name: &token::Token, value: &expr::Expr) -> String {
        format!(
            "(= {} {} {}",
            print_expr(object),
            name.lexeme,
            print_expr(value)
        )
    }

    fn print_expr_super(_keyword: &token::Token, method: &token::Token) -> String {
        format!("(super {})", method.lexeme,)
    }

    fn print_expr_this() -> String {
        "this".to_string()
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
    // this is used in parser and interpreter tests
}
