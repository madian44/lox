use crate::location;

pub trait Reporter {
    fn add_diagnostic(
        &mut self,
        start: &location::FileLocation,
        end: &location::FileLocation,
        message: &str,
    );

    fn add_message(&mut self, message: &str);

    fn has_diagnostics(&self) -> bool;
}
