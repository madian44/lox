mod location;
mod reporter;
mod scanner;
mod token;

pub use crate::location::FileLocation;
pub use crate::reporter::Reporter;

pub fn run<'s>(reporter: &mut dyn reporter::Reporter, source: &'s str) -> Vec<token::Token<'s>> {
    scanner::scan_tokens(reporter, source)
}
