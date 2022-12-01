#[cfg(not(target_arch = "wasm32"))]
mod app;
#[cfg(not(target_arch = "wasm32"))]
mod cross;
#[cfg(not(target_arch = "wasm32"))]
mod gen;
#[cfg(not(target_arch = "wasm32"))]
mod introspection;
#[cfg(not(target_arch = "wasm32"))]
mod run;
#[cfg(not(target_arch = "wasm32"))]
mod typescript;
#[cfg(not(target_arch = "wasm32"))]
mod util;

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> eyre::Result<()> {
    run::run().await
}

#[cfg(target_arch = "wasm32")]
fn main() {}
