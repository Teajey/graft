use std::collections::HashMap;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen]
    type Process;

    #[allow(non_upper_case_globals)]
    static PROCESS: Process;

    #[wasm_bindgen(method, getter)]
    fn argv(this: &Process) -> Vec<JsValue>;

    #[wasm_bindgen(js_namespace = console)]
    pub fn log(arg: &str);

    #[wasm_bindgen(js_namespace = console)]
    pub fn error(arg: &str);

    #[wasm_bindgen(js_name = "process.stdout.write")]
    pub fn process_stdout_write(arg: &str);

    #[wasm_bindgen(js_name = "process.stderr.write")]
    pub fn process_stderr_write(arg: &str);

    #[wasm_bindgen(js_name = "process.chdir", catch)]
    pub fn process_chdir(path: &str) -> Result<(), JsValue>;

    #[wasm_bindgen(js_name = "process.cwd", catch)]
    pub fn process_cwd() -> Result<String, JsValue>;

    #[wasm_bindgen(js_name = "process.exit")]
    pub fn process_exit(code: i32);

    #[wasm_bindgen(js_name = "process.argv")]
    pub fn process_argv() -> Vec<JsValue>;

    #[wasm_bindgen(method, getter)]
    pub fn env(this: &Process) -> JsValue;
}

pub fn process_env() -> HashMap<String, String> {
    serde_wasm_bindgen::from_value(PROCESS.env())
        .expect("process.env must be HashMap<String, String>")
}

#[wasm_bindgen(module = "fs")]
extern "C" {
    #[wasm_bindgen(js_name = writeFileSync, catch)]
    pub fn write_file(path: &str, data: &str) -> Result<(), JsValue>;

    #[wasm_bindgen(js_name = readdirSync, catch)]
    pub fn read_dir(path: &str) -> Result<Vec<JsValue>, JsValue>;

    // #[wasm_bindgen(js_name = existsSync, catch)]
    // pub fn path_try_exists(path: &str) -> Result<bool, JsValue>;
}

#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => ($crate::cross::node::log(&format_args!($($t)*).to_string()))
}

#[macro_export]
macro_rules! console_error {
    ($($t:tt)*) => ($crate::cross::node::error(&format_args!($($t)*).to_string()))
}

#[macro_export]
macro_rules! node_stdout {
    ($($t:tt)*) => ($crate::cross::node::process_stdout_write(&format_args!($($t)*).to_string()))
}

#[macro_export]
macro_rules! node_stderr {
    ($($t:tt)*) => ($crate::cross::node::process_stderr_write(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen(module = "/node.js")]
extern "C" {
    #[wasm_bindgen(js_name = "fetchJson", catch)]
    pub async fn fetch_json(url: &str, no_ssl: bool, options: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(js_name = "readFileToString", catch)]
    pub fn read_file_to_string(path: &str) -> Result<String, JsValue>;
}
