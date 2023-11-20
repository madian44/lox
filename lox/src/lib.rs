mod ast_printer;
mod expr;
mod location;
mod reporter;
mod scanner;
mod token;

pub use crate::location::FileLocation;
pub use crate::reporter::Reporter;

pub fn run(reporter: &mut dyn reporter::Reporter, source: &str) -> Vec<token::Token> {
    scanner::scan_tokens(reporter, source)
}
