use lox;
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "vscode")]
extern "C" {
    type Window;

    #[wasm_bindgen(js_namespace=["window"], js_name=showInformationMessage)]
    fn showInformationMessage(s: &str);
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn console_log(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    showInformationMessage("Hello, wasm");
}

#[derive(Serialize)]
struct FileLocation {
    line_number: u32,
    line_offset: u32,
}

type MessageReporter = Box<dyn Fn(&str)>;
type DiagnosticReporter = Box<dyn Fn(&lox::FileLocation, &lox::FileLocation, &str)>;

struct WasmReporter {
    message_reporter: MessageReporter,
    diagnostic_reporter: DiagnosticReporter,
    has_errors: bool,
}

impl WasmReporter {
    fn build(message_reporter: MessageReporter, diagnostic_reporter: DiagnosticReporter) -> Self {
        WasmReporter {
            message_reporter,
            diagnostic_reporter,
            has_errors: false,
        }
    }
}

impl lox::Reporter for WasmReporter {
    fn add_diagnostic(
        &mut self,
        start: &lox::FileLocation,
        end: &lox::FileLocation,
        message: &str,
    ) {
        self.has_errors = true;
        (self.diagnostic_reporter)(start, end, message);
    }

    fn add_message(&mut self, message: &str) {
        (self.message_reporter)(message);
    }

    fn has_diagnostics(&self) -> bool {
        self.has_errors
    }
}

impl FileLocation {
    fn build(other: &lox::FileLocation) -> Self {
        FileLocation {
            line_number: other.line_number,
            line_offset: other.line_offset,
        }
    }
}

#[wasm_bindgen]
pub fn scan(
    text: &str,
    js_report_message: js_sys::Function,
    js_report_diagnostic: js_sys::Function,
) {
    let this_message = JsValue::null();
    let this_diagnostic = JsValue::null();
    let report_message = Box::new(move |message: &str| {
        let _ = js_report_message.call1(&this_message, &JsValue::from(message));
    });

    let report_diagnostic = Box::new(
        move |start: &lox::FileLocation, end: &lox::FileLocation, message: &str| {
            let _ = js_report_diagnostic.call3(
                &this_diagnostic,
                &serde_wasm_bindgen::to_value(&FileLocation::build(start)).unwrap(),
                &serde_wasm_bindgen::to_value(&FileLocation::build(end)).unwrap(),
                &JsValue::from(message),
            );
        },
    );

    let mut reporter = WasmReporter::build(report_message, report_diagnostic);

    console_log(&format!("scanning {text}"));
    lox::run(&mut reporter, text);
}
