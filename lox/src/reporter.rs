type MessageReporter = Box<dyn Fn(&str)>;
type DiagnosticReporter = Box<dyn Fn(u32, u32, &str)>;

pub struct Reporter {
    message_reporter: MessageReporter,
    diagnostic_reporter: DiagnosticReporter,
}

impl Reporter {
    pub fn build(
        message_reporter: MessageReporter,
        diagnostic_reporter: DiagnosticReporter,
    ) -> Self {
        Reporter {
            message_reporter,
            diagnostic_reporter,
        }
    }

    pub fn add_message(&self, message: &str) {
        (self.message_reporter)(message);
    }

    pub fn add_diagnostic(&self, line: u32, column: u32, message: &str) {
        (self.diagnostic_reporter)(line, column, message);
    }
}
