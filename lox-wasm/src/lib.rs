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
pub fn scan(text: &str) -> Result<String, JsValue> {
    console_log(&format!("scanning {text}"));
    let output = lox::run(text);
    match output {
        Ok(s) => {
            console_log(&format!("Success: {s}"));
            Ok(s)
        }
        Err(s) => {
            console_log(&format!("Error: {s}"));
            Err(JsValue::from_str(s))
        }
    }
}
