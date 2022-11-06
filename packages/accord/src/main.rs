mod cli;
mod common;
mod config;
mod gen;
mod introspection;
mod typescript;
mod util;

use std::io::Write;

use clap::Parser;
use eyre::Result;

use crate::gen::generate_typescript;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = cli::Base::parse();

    if let Some(working_dir) = &cli.working_directory {
        std::env::set_current_dir(working_dir)?;
    }

    let config = config::load(cli.config_location.as_deref()).unwrap_or_else(|err| {
        eprintln!("Failed to load config: {}", err);
        std::process::exit(1);
    });

    let ts = generate_typescript(cli, config).await?;

    write!(std::fs::File::create("generated.ts")?, "{ts}")?;

    Ok(())
}
