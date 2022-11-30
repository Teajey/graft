#[cfg(not(target_arch = "wasm32"))]
mod run;

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> eyre::Result<()> {
    run::run().await
}

#[cfg(target_arch = "wasm32")]
fn main() {}
