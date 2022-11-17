mod cli;
mod config;
mod gen;
mod introspection;
#[cfg(feature = "native")]
mod native;
#[cfg(feature = "node")]
mod node;
mod typescript;
mod util;

#[cfg(any(
    all(feature = "node", feature = "native"),
    not(any(feature = "node", feature = "native"))
))]
compile_error!(r#"The "node" OR the "native" feature should be selected."#);

#[cfg(feature = "node")]
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), wasm_bindgen::JsValue> {
    node::main::node_main().await
}

#[cfg(feature = "native")]
use eyre::Result;

#[cfg(feature = "native")]
#[tokio::main]
async fn main() -> Result<()> {
    use std::io::Write;

    use clap::Parser;

    use crate::native::gen::generate_typescript;

    let cli = cli::Base::parse();

    if let Some(working_dir) = &cli.working_directory {
        std::env::set_current_dir(working_dir)?;
    }

    let config = native::config::load(cli.config_location.as_deref()).unwrap_or_else(|err| {
        eprintln!("Failed to load config: {}", err);
        std::process::exit(1);
    });

    let config = config::AppConfig::try_from(config)?;

    let ts = generate_typescript(cli, config).await?;

    write!(std::fs::File::create("generated.ts")?, "{ts}")?;

    Ok(())
}
