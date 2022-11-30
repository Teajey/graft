mod cli;
mod config;
mod cross;
mod gen;
mod introspection;
mod run;
mod typescript;
mod util;

use eyre::Result;
use wasm_bindgen::prelude::*;

use run::run;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub async fn node_main() -> Result<(), JsValue> {
    run()
        .await
        .map_err(|err| JsValue::from_str(&err.to_string()))
}
