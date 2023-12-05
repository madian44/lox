use crate::{expr, token};

pub enum Stmt {
    Expression {
        expression: expr::Expr,
    },
    Print {
        value: expr::Expr,
    },
    Var {
        name: token::Token,
        initialiser: Option<expr::Expr>,
    },
}
