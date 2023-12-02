#[derive(Debug, PartialEq)]
pub struct Diagnostic {
    pub start: lox::FileLocation,
    pub end: lox::FileLocation,
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

    pub fn has_message(&self, message_to_search_for: &str) -> bool {
        self.messages
            .iter()
            .any(|m| m.message == message_to_search_for)
    }

    pub fn has_diagnostic(&self, diagnostic_to_search_for: &Diagnostic) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d == diagnostic_to_search_for)
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

impl lox::Reporter for TestReporter {
    fn add_diagnostic(
        &mut self,
        start: &lox::FileLocation,
        end: &lox::FileLocation,
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
