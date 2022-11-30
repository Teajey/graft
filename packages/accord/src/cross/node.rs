use std::collections::HashMap;

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

    #[wasm_bindgen(js_namespace = console)]
    pub fn log(arg: &str);

    #[wasm_bindgen(js_name = "process.stdout.write")]
    pub fn process_stdout_write(arg: &str);

    #[wasm_bindgen(js_name = "process.chdir", catch)]
    pub fn process_chdir(path: &str) -> Result<(), JsValue>;

    #[wasm_bindgen(method, getter)]
    pub fn env(this: &Process) -> JsValue;
}

pub fn process_env() -> HashMap<String, String> {
    serde_wasm_bindgen::from_value(process.env())
        .expect("process.env must be HashMap<String, String>")
}

#[wasm_bindgen(module = "fs")]
extern "C" {
    #[wasm_bindgen(js_name = readFileSync, catch)]
    pub fn read_file(path: &str) -> Result<Buffer, JsValue>;

    #[wasm_bindgen(js_name = writeFileSync, catch)]
    pub fn write_file(path: &str, data: &str) -> Result<(), JsValue>;

    // #[wasm_bindgen(js_name = existsSync, catch)]
    // pub fn path_try_exists(path: &str) -> Result<bool, JsValue>;
}

#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => ($crate::cross::node::log(&format_args!($($t)*).to_string()))
}

#[macro_export]
macro_rules! node_stdout {
    ($($t:tt)*) => (node::process_stdout_write(&format_args!($($t)*).to_string()))
}

pub fn argv() -> Vec<JsValue> {
    process.argv()
}

#[wasm_bindgen(module = "/fetchJson.js")]
extern "C" {
    #[wasm_bindgen(js_name = "fetchJson", catch)]
    pub async fn fetch_json(url: &str, no_ssl: bool, options: JsValue) -> Result<JsValue, JsValue>;
}
