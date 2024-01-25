use crate::{expr, location, token};
use std::sync::atomic::{AtomicUsize, Ordering};

static ID_SRC: AtomicUsize = AtomicUsize::new(1);

#[derive(Debug)]
pub enum Expr {
    Assign {
        id: usize,
        name: token::Token,
        value: Box<Expr>,
    },
    Binary {
        id: usize,
        left: Box<Expr>,
        operator: token::Token,
        right: Box<Expr>,
    },
    Call {
        id: usize,
        callee: Box<Expr>,
        paren: token::Token,
        arguments: Vec<Expr>,
    },
    Get {
        id: usize,
        object: Box<Expr>,
        name: token::Token,
    },
    Grouping {
        id: usize,
        expression: Box<Expr>,
    },
    Literal {
        id: usize,
        value: token::Token,
    },
    Logical {
        id: usize,
        left: Box<Expr>,
        operator: token::Token,
        right: Box<Expr>,
    },
    Set {
        id: usize,
        object: Box<Expr>,
        name: token::Token,
        value: Box<Expr>,
    },
    Super {
        id: usize,
        keyword: token::Token,
        method: token::Token,
    },
    This {
        id: usize,
        keyword: token::Token,
    },
    Unary {
        id: usize,
        operator: token::Token,
        right: Box<Expr>,
    },
    Variable {
        id: usize,
        name: token::Token,
    },
}

impl Expr {
    fn get_id() -> usize {
        ID_SRC.fetch_add(1, Ordering::Relaxed)
    }

    pub fn new_assign(name: token::Token, value: Expr) -> Self {
        Expr::Assign {
            id: Expr::get_id(),
            name,
            value: Box::new(value),
        }
    }

    pub fn new_binary(left: Expr, operator: token::Token, right: Expr) -> Self {
        Expr::Binary {
            id: Expr::get_id(),
            left: Box::new(left),
            operator,
            right: Box::new(right),
        }
    }

    pub fn new_call(callee: Expr, paren: token::Token, arguments: Vec<Expr>) -> Self {
        Expr::Call {
            id: Expr::get_id(),
            callee: Box::new(callee),
            paren,
            arguments,
        }
    }

    pub fn new_get(object: Expr, name: token::Token) -> Self {
        Expr::Get {
            id: Expr::get_id(),
            object: Box::new(object),
            name,
        }
    }

    pub fn new_grouping(expression: Expr) -> Self {
        Expr::Grouping {
            id: Expr::get_id(),
            expression: Box::new(expression),
        }
    }

    pub fn new_literal(value: token::Token) -> Self {
        Expr::Literal {
            id: Expr::get_id(),
            value,
        }
    }

    pub fn new_logical(left: Expr, operator: token::Token, right: Expr) -> Self {
        Expr::Logical {
            id: Expr::get_id(),
            left: Box::new(left),
            operator,
            right: Box::new(right),
        }
    }

    pub fn new_set(object: Expr, name: token::Token, value: Expr) -> Self {
        Expr::Set {
            id: Expr::get_id(),
            object: Box::new(object),
            name,
            value: Box::new(value),
        }
    }

    pub fn new_super(keyword: token::Token, method: token::Token) -> Self {
        Expr::Super {
            id: Expr::get_id(),
            keyword,
            method,
        }
    }

    pub fn new_this(keyword: token::Token) -> Self {
        Expr::This {
            id: Expr::get_id(),
            keyword,
        }
    }

    pub fn new_unary(operator: token::Token, right: Expr) -> Self {
        Expr::Unary {
            id: Expr::get_id(),
            operator,
            right: Box::new(right),
        }
    }

    pub fn new_variable(name: token::Token) -> Self {
        Expr::Variable {
            id: Expr::get_id(),
            name,
        }
    }
}

fn get_start_location(expr: &expr::Expr) -> &location::FileLocation {
    match expr {
        expr::Expr::Assign { name, .. } => &name.start,
        expr::Expr::Binary { left, .. } => get_start_location(left),
        expr::Expr::Call { callee, .. } => get_start_location(callee),
        expr::Expr::Get { object, .. } => get_start_location(object),
        expr::Expr::Grouping { expression, .. } => get_start_location(expression),
        expr::Expr::Literal { value, .. } => &value.start,
        expr::Expr::Logical { left, .. } => get_start_location(left),
        expr::Expr::Set { object, .. } => get_start_location(object),
        expr::Expr::Super { keyword, .. } => &keyword.start,
        expr::Expr::This { keyword, .. } => &keyword.start,
        expr::Expr::Unary { operator, .. } => &operator.start,
        expr::Expr::Variable { name, .. } => &name.start,
    }
}

fn get_end_location(expr: &expr::Expr) -> &location::FileLocation {
    match expr {
        expr::Expr::Assign { value, .. } => get_end_location(value),
        expr::Expr::Binary { right, .. } => get_end_location(right),
        expr::Expr::Call { paren, .. } => &paren.end,
        expr::Expr::Grouping { expression, .. } => get_end_location(expression),
        expr::Expr::Get { name, .. } => &name.end,
        expr::Expr::Literal { value, .. } => &value.end,
        expr::Expr::Logical { right, .. } => get_end_location(right),
        expr::Expr::Set { value, .. } => get_end_location(value),
        expr::Expr::Super { method, .. } => &method.end,
        expr::Expr::This { keyword, .. } => &keyword.end,
        expr::Expr::Unary { right, .. } => get_end_location(right),
        expr::Expr::Variable { name, .. } => &name.end,
    }
}

impl location::ProvideLocation for Expr {
    fn start(&self) -> &location::FileLocation {
        get_start_location(self)
    }

    fn end(&self) -> &location::FileLocation {
        get_end_location(self)
    }
}

impl<'a> location::ProvideLocation for &'a Expr {
    fn start(&self) -> &location::FileLocation {
        get_start_location(self)
    }

    fn end(&self) -> &location::FileLocation {
        get_end_location(self)
    }
}
