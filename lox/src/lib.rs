pub mod reporter;

mod location;
mod scanner;
mod token;

pub use crate::location::FileLocation;

pub fn run(reporter: &mut dyn reporter::Reporter, source: &str) {
    scanner::scan_tokens(reporter, source);

    // reporter.add_diagnostic(
    //     &FileLocation {
    //         line_number: 1,
    //         line_offset: 1,
    //     },
    //     &FileLocation {
    //         line_number: 1,
    //         line_offset: 10,
    //     },
    //     "A diagnostic between 1 and 10",
    // );
    //
    // reporter.add_message("A message saying 'Hello' ");
}
