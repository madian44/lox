use crate::location;

pub trait Reporter {
    fn add_diagnostic(
        &self,
        start: &location::FileLocation,
        end: &location::FileLocation,
        message: &str,
    );

    fn add_message(&self, message: &str);

    fn has_diagnostics(&self) -> bool;
}

#[cfg(test)]
pub mod test {

    use super::*;
    use crate::location;
    use std::cell::RefCell;

    #[derive(Debug, PartialEq, Clone)]
    pub struct Diagnostic {
        pub start: location::FileLocation,
        pub end: location::FileLocation,
        pub message: String,
    }

    #[derive(Debug)]
    pub struct Message {
        pub message: String,
    }

    pub struct TestReporter {
        diagnostics: RefCell<Vec<Diagnostic>>,
        messages: RefCell<Vec<Message>>,
    }

    impl TestReporter {
        pub fn build() -> Self {
            TestReporter {
                diagnostics: RefCell::new(Vec::new()),
                messages: RefCell::new(Vec::new()),
            }
        }

        pub fn has_messages(&self) -> bool {
            !self.messages.borrow().is_empty()
        }

        pub fn diagnostics_len(&self) -> usize {
            self.diagnostics.borrow().len()
        }

        pub fn diagnostic_get(&self, index: usize) -> Option<Diagnostic> {
            self.diagnostics.borrow().get(index).cloned()
        }

        pub fn has_message(&self, message: &str) -> bool {
            self.messages.borrow().iter().any(|m| m.message == message)
        }
        
        pub fn has_diagnostic(&self, message: &str) -> bool {
            self.diagnostics.borrow().iter().any(|m| m.message == message)
        }

        pub fn reset(&self) {
            self.messages.borrow_mut().clear();
            self.diagnostics.borrow_mut().clear();
        }

        pub fn print_contents(&self) {
            self.messages
                .borrow()
                .iter()
                .for_each(|m| println!("[message] {m:?}"));
            self.diagnostics
                .borrow()
                .iter()
                .for_each(|d| println!("[diagnostic] {d:?}"))
        }
    }

    impl Reporter for TestReporter {
        fn add_diagnostic(
            &self,
            start: &location::FileLocation,
            end: &location::FileLocation,
            message: &str,
        ) {
            self.diagnostics.borrow_mut().push(Diagnostic {
                start: *start,
                end: *end,
                message: message.to_string(),
            });
        }

        fn add_message(&self, message: &str) {
            self.messages.borrow_mut().push(Message {
                message: message.to_string(),
            });
        }
        fn has_diagnostics(&self) -> bool {
            !self.diagnostics.borrow().is_empty()
        }
    }
}
