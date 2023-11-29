mod ast_printer;
mod expr;
mod interpreter;
mod location;
mod lox_type;
mod parser;
mod reporter;
mod scanner;
mod token;

pub use crate::location::FileLocation;
pub use crate::reporter::Reporter;

pub fn run(reporter: &mut dyn reporter::Reporter, source: &str) {
    interpret(reporter, source);
}

pub fn scan(reporter: &mut dyn reporter::Reporter, source: &str) {
    let tokens = scanner::scan_tokens(reporter, source);
    tokens
        .iter()
        .for_each(|t| reporter.add_message(&format!("[token]: {t}")));
}

pub fn parse(reporter: &mut dyn reporter::Reporter, source: &str) {
    let tokens = scanner::scan_tokens(reporter, source);
    tokens
        .iter()
        .for_each(|t| reporter.add_message(&format!("[token]: {t}")));
    if reporter.has_diagnostics() {
        reporter.add_message("[parser] not parsing due to scan errors");
        return;
    }
    if let Some(expr) = parser::parse(reporter, tokens) {
        reporter.add_message(&format!("[expr] {}", ast_printer::print(&expr)));
    }
}

pub fn interpret(reporter: &mut dyn reporter::Reporter, source: &str) {
    let tokens = scanner::scan_tokens(reporter, source);
    tokens
        .iter()
        .for_each(|t| reporter.add_message(&format!("[token]: {t}")));
    if reporter.has_diagnostics() {
        reporter.add_message("[parser] not parsing due to scan errors");
        return;
    }

    let parse = parser::parse(reporter, tokens);

    if reporter.has_diagnostics() {
        reporter.add_message("[interpreter] not interpreting due to parsing errors");
        return;
    }

    if let Some(expr) = parse {
        reporter.add_message(&format!("[expr] {}", ast_printer::print(&expr)));

        if let Some(lox_type) = interpreter::interpret(reporter, &expr) {
            reporter.add_message(&format!("[interpreter] {}", lox_type));
        }
    }
}
