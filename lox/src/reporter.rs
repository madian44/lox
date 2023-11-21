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

#[cfg(test)]
pub mod test {

    use super::*;
    use crate::location;

    #[derive(Debug, PartialEq)]
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
        pub diagnostics: Vec<Diagnostic>,
        pub messages: Vec<Message>,
    }

    impl TestReporter {
        pub fn build() -> Self {
            TestReporter {
                diagnostics: Vec::new(),
                messages: Vec::new(),
            }
        }

        pub fn has_messages(&self) -> bool {
            !self.messages.is_empty()
        }

        pub fn reset(&mut self) {
            self.messages.clear();
            self.diagnostics.clear();
        }

        pub fn print_contents(&self) {
            self.messages
                .iter()
                .for_each(|m| println!("[message] {m:?}"));
            self.diagnostics
                .iter()
                .for_each(|d| println!("[diagnostic] {d:?}"))
        }
    }

    impl Reporter for TestReporter {
        fn add_diagnostic(
            &mut self,
            start: &location::FileLocation,
            end: &location::FileLocation,
            message: &str,
        ) {
            self.diagnostics.push(Diagnostic {
                start: *start,
                end: *end,
                message: message.to_string(),
            });
        }

        fn add_message(&mut self, message: &str) {
            self.messages.push(Message {
                message: message.to_string(),
            });
        }
        fn has_diagnostics(&self) -> bool {
            !self.diagnostics.is_empty()
        }
    }
}
