#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() -> eyre::Result<()> {
    teajey_graft::run().await
}
