mod language;

use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::convert;
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

#[derive(Serialize, Deserialize, Debug)]
struct FileLocation {
    line_number: u32,
    line_offset: u32,
}

#[derive(Serialize, Deserialize, Debug)]
struct Range {
    start: FileLocation,
    end: FileLocation,
}

#[derive(Serialize, Deserialize, Debug)]
struct Location {
    path: String,
    range: Range,
}

#[derive(Serialize, Debug)]
struct Completion {
    name: String,
    completion_type: u32,
}

type MessageReporter = Box<dyn Fn(&str)>;
type DiagnosticReporter = Box<dyn Fn(&lox::FileLocation, &lox::FileLocation, &str)>;

struct WasmReporter {
    message_reporter: MessageReporter,
    diagnostic_reporter: DiagnosticReporter,
    has_errors: RefCell<bool>,
}

impl WasmReporter {
    fn build(message_reporter: MessageReporter, diagnostic_reporter: DiagnosticReporter) -> Self {
        WasmReporter {
            message_reporter,
            diagnostic_reporter,
            has_errors: RefCell::new(false),
        }
    }
}

impl lox::Reporter for WasmReporter {
    fn add_diagnostic(&self, start: &lox::FileLocation, end: &lox::FileLocation, message: &str) {
        *self.has_errors.borrow_mut() = true;
        (self.diagnostic_reporter)(start, end, message);
    }

    fn add_message(&self, message: &str) {
        (self.message_reporter)(message);
    }

    fn has_diagnostics(&self) -> bool {
        *self.has_errors.borrow()
    }
}

impl FileLocation {
    fn new(other: &lox::FileLocation) -> Self {
        FileLocation {
            line_number: other.line_number,
            line_offset: other.line_offset,
        }
    }
}

impl convert::From<lox::FileLocation> for FileLocation {
    fn from(value: lox::FileLocation) -> Self {
        FileLocation {
            line_offset: value.line_offset,
            line_number: value.line_number,
        }
    }
}

impl convert::From<FileLocation> for lox::FileLocation {
    fn from(value: FileLocation) -> Self {
        lox::FileLocation {
            line_offset: value.line_offset,
            line_number: value.line_number,
        }
    }
}

#[wasm_bindgen]
pub fn scan(
    text: &str,
    js_report_message: js_sys::Function,
    js_report_diagnostic: js_sys::Function,
) {
    let reporter = build_reporter(js_report_message, js_report_diagnostic);

    console_log(&format!("scanning: {text}"));
    lox::scan(&reporter, text);
}

#[wasm_bindgen]
pub fn parse(
    text: &str,
    js_report_message: js_sys::Function,
    js_report_diagnostic: js_sys::Function,
) {
    let reporter = build_reporter(js_report_message, js_report_diagnostic);

    console_log(&format!("parsing: {text}"));
    lox::parse(&reporter, text);
}

#[wasm_bindgen]
pub fn resolve(
    text: &str,
    js_report_message: js_sys::Function,
    js_report_diagnostic: js_sys::Function,
) {
    let reporter = build_reporter(js_report_message, js_report_diagnostic);

    console_log(&format!("resolving: {text}"));
    lox::resolve(&reporter, text);
}

#[wasm_bindgen]
pub fn interpret(
    text: &str,
    js_report_message: js_sys::Function,
    js_report_diagnostic: js_sys::Function,
) {
    let reporter = build_reporter(js_report_message, js_report_diagnostic);

    console_log(&format!("interpreting: {text}"));
    lox::interpret(&reporter, text);
}

fn build_reporter(
    js_report_message: js_sys::Function,
    js_report_diagnostic: js_sys::Function,
) -> WasmReporter {
    let this_message = JsValue::null();
    let this_diagnostic = JsValue::null();
    let report_message = Box::new(move |message: &str| {
        let _ = js_report_message.call1(&this_message, &JsValue::from(message));
    });

    let report_diagnostic = Box::new(
        move |start: &lox::FileLocation, end: &lox::FileLocation, message: &str| {
            let _ = js_report_diagnostic.call3(
                &this_diagnostic,
                &serde_wasm_bindgen::to_value(&FileLocation::new(start)).unwrap(),
                &serde_wasm_bindgen::to_value(&FileLocation::new(end)).unwrap(),
                &JsValue::from(message),
            );
        },
    );

    WasmReporter::build(report_message, report_diagnostic)
}

#[wasm_bindgen]
pub fn provide_definition(contents: &str, path: &str, position: JsValue) -> Box<[JsValue]> {
    let position: FileLocation = serde_wasm_bindgen::from_value(position).unwrap();

    let definitions = language::provide_definition(&lox::FileLocation::from(position), contents);

    let result = definitions
        .iter()
        .map(|t| Location {
            path: path.to_string(),
            range: Range {
                start: FileLocation::from(t.start),
                end: FileLocation::from(t.end),
            },
        })
        .map(|l| serde_wasm_bindgen::to_value(&l).unwrap())
        .collect::<Vec<JsValue>>();

    result.into_boxed_slice()
}

#[wasm_bindgen]
pub fn provide_completions(contents: &str, position: JsValue) -> Box<[JsValue]> {
    let position: FileLocation = serde_wasm_bindgen::from_value(position).unwrap();

    let completions = language::provide_completions(&lox::FileLocation::from(position), contents);

    completions
        .into_iter()
        .map(|(name, completion_type)| Completion {
            name,
            completion_type,
        })
        .map(|c| serde_wasm_bindgen::to_value(&c).unwrap())
        .collect::<Vec<JsValue>>()
        .into_boxed_slice()
}
