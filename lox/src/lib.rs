mod ast_printer;
mod expr;
mod interpreter;
mod location;
mod parser;
mod reporter;
mod resolver;
mod scanner;
mod stmt;
mod token;

pub use crate::expr::Expr;
pub use crate::location::FileLocation;
pub use crate::reporter::Reporter;
pub use crate::stmt::function::Function;
pub use crate::stmt::Stmt;
pub use crate::token::Token;
pub use crate::token::TokenType;

use std::collections::LinkedList;

pub fn run(reporter: &dyn reporter::Reporter, source: &str) {
    interpret(reporter, source);
}

pub fn scan(reporter: &dyn reporter::Reporter, source: &str) {
    let tokens = scanner::scan_tokens(reporter, source);
    tokens
        .iter()
        .for_each(|t| reporter.add_message(&format!("[token]: {t}")));
}

pub fn parse(reporter: &dyn reporter::Reporter, source: &str) {
    let tokens = scanner::scan_tokens(reporter, source);
    tokens
        .iter()
        .for_each(|t| reporter.add_message(&format!("[token]: {t}")));
    if reporter.has_diagnostics() {
        reporter.add_message("[parser] not parsing due to scan errors");
        return;
    }
    for stmt in &parser::parse(reporter, tokens) {
        reporter.add_message(&format!("[stmt] {}", ast_printer::print_stmt(stmt)));
    }
}

pub fn resolve(reporter: &dyn reporter::Reporter, source: &str) {
    let tokens = scanner::scan_tokens(reporter, source);
    if reporter.has_diagnostics() {
        reporter.add_message("[resolve] not parsing due to scan errors");
        return;
    }
    let statements = parser::parse(reporter, tokens);
    if reporter.has_diagnostics() {
        reporter.add_message("[resolve] not resolve due to parsing errors");
        return;
    }

    let _ = resolver::resolve(reporter, &statements);
}

pub fn interpret(reporter: &dyn reporter::Reporter, source: &str) {
    let tokens = scanner::scan_tokens(reporter, source);
    if reporter.has_diagnostics() {
        reporter.add_message("[parser] not parsing due to scan errors");
        return;
    }

    let statements = parser::parse(reporter, tokens);
    if reporter.has_diagnostics() {
        reporter.add_message("[interpreter] not interpreting due to parsing errors");
        return;
    }

    let depths = resolver::resolve(reporter, &statements);
    if reporter.has_diagnostics() {
        reporter.add_message("[interpreter] not interpreting due to resolver errors");
        return;
    }

    interpreter::interpret(reporter, &depths, statements);
}

pub fn ast(reporter: &dyn reporter::Reporter, source: &str) -> LinkedList<stmt::Stmt> {
    let tokens = scanner::scan_tokens(reporter, source);
    if reporter.has_diagnostics() {
        return LinkedList::new();
    }
    parser::parse(reporter, tokens)
}
