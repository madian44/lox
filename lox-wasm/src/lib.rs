
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module="vscode")]
extern "C" {

    type Window;

    #[wasm_bindgen(js_namespace=["window"], js_name=showInformationMessage)]
    fn showInformationMessage(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    showInformationMessage("Hello, wasm");
}

#[wasm_bindgen]
pub fn scan(text: &str) -> String {
    showInformationMessage("Hello, wasm");
    format!("output of a scan {text}")
}
