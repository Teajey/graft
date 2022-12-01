#[cfg(target_arch = "wasm32")]
mod app;
#[cfg(target_arch = "wasm32")]
mod cross;
#[cfg(target_arch = "wasm32")]
mod gen;
#[cfg(target_arch = "wasm32")]
mod introspection;
#[cfg(target_arch = "wasm32")]
mod run;
#[cfg(target_arch = "wasm32")]
mod typescript;
#[cfg(target_arch = "wasm32")]
mod util;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub async fn node_main() -> eyre::Result<(), wasm_bindgen::JsValue> {
    run::run()
        .await
        .map_err(|err| wasm_bindgen::JsValue::from_str(&err.to_string()))
}
