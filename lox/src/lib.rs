mod ast_printer;
mod expr;
mod location;
mod parser;
mod reporter;
mod scanner;
mod token;

pub use crate::location::FileLocation;
pub use crate::reporter::Reporter;

pub fn run(reporter: &mut dyn reporter::Reporter, source: &str) {
    parse(reporter, source);
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
