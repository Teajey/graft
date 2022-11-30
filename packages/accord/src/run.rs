use clap::Parser;
use eyre::Result;

use crate::gen::generate_typescript;
use crate::introspection;
use crate::{cli, config::AppConfig, cross};

pub async fn run() -> Result<()> {
    let argv: Result<Vec<_>> = cross::env::argv().collect();

    let cli = cli::Base::try_parse_from(argv?)?;

    if let Some(working_dir) = &cli.working_directory {
        cross::env::set_current_dir(working_dir)?;
    }

    let config = AppConfig::load(cli.config_location.as_deref()).unwrap_or_else(|err| {
        eprintln!("Failed to load config: {}", err);
        std::process::exit(1);
    });

    let schema = introspection::Response::fetch(&config).await?.schema();

    let ts = generate_typescript(cli, config, schema).await?;

    cross::fs::write_to_file("generated.ts", &ts)?;

    Ok(())
}
