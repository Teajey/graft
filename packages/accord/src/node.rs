use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    type Process;

    pub type Buffer;

    #[allow(non_upper_case_globals)]
    static process: Process;

    #[wasm_bindgen(method, getter)]
    fn argv(this: &Process) -> Vec<JsValue>;
}

#[wasm_bindgen(module = "fs")]
extern "C" {
    #[wasm_bindgen(js_name = readFileSync, catch)]
    pub fn read_file(path: &str) -> Result<Buffer, JsValue>;

    #[wasm_bindgen(js_name = readFileSync, catch)]
    pub fn write_file(path: &str, data: &str) -> Result<(), JsValue>;
}

pub fn argv() -> Vec<JsValue> {
    process.argv()
}
