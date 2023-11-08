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

#[wasm_bindgen]
pub fn scan(
    text: &str,
    js_report_message: js_sys::Function,
    js_report_diagnostic: js_sys::Function,
) {
    let this1 = JsValue::null();
    let this2 = JsValue::null();
    let report_message = Box::new(move |message: &str| {
        let _ = js_report_message.call1(&this1, &JsValue::from(message));
    });

    let report_diagnostic = Box::new(move |line: u32, column: u32, message: &str| {
        let _ = js_report_diagnostic.call3(
            &this2,
            &JsValue::from(line),
            &JsValue::from(column),
            &JsValue::from(message),
        );
    });

    let reporter = lox::reporter::Reporter::build(report_message, report_diagnostic);

    console_log(&format!("scanning {text}"));
    lox::run(&reporter, text);
}
