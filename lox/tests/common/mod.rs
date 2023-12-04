use std::cell::RefCell;

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
    pub diagnostics: RefCell<Vec<Diagnostic>>,
    pub messages: RefCell<Vec<Message>>,
}

impl TestReporter {
    pub fn build() -> Self {
        TestReporter {
            diagnostics: RefCell::new(Vec::new()),
            messages: RefCell::new(Vec::new()),
        }
    }

    pub fn has_message(&self, message_to_search_for: &str) -> bool {
        self.messages
            .borrow()
            .iter()
            .any(|m| m.message == message_to_search_for)
    }

    pub fn has_diagnostic(&self, diagnostic_to_search_for: &Diagnostic) -> bool {
        self.diagnostics
            .borrow()
            .iter()
            .any(|d| d == diagnostic_to_search_for)
    }

    pub fn reset(&mut self) {
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

impl lox::Reporter for TestReporter {
    fn add_diagnostic(&self, start: &lox::FileLocation, end: &lox::FileLocation, message: &str) {
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
