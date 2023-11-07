
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
