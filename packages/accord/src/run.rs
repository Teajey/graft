use clap::Parser;
use eyre::{eyre, Result};

use crate::gen::generate_typescript;
use crate::{
    app::{self, cli},
    cross, util,
};
use crate::{cross_eprint, cross_eprintln, introspection, print_info};

pub async fn run() -> Result<()> {
    util::debug::log("Collecting argv\n")?;
    let argv: Result<Vec<_>> = cross::env::argv().collect();

    util::debug::log("Parsing cli\n")?;
    let cli = match cli::Base::try_parse_from(argv?) {
        Ok(cli) => cli,
        Err(err) => {
            cross_eprint!("{err}");
            cross::process::exit(0);
        }
    };

    if let Some(working_dir) = &cli.working_directory {
        util::debug::log("Setting current directory\n")?;
        cross::env::set_current_dir(working_dir)?;
    }

    let config_location = match cli.config_location.clone() {
        Some(config_location) => Some(config_location),
        None => {
            util::debug::log("Searching for config\n")?;
            match util::file_location_in_path_by_prefix(".accord")?.map(|path| {
                path.to_str()
                    .map(|string| string.to_owned())
                    .ok_or_else(|| eyre!("Couldn't convert config location pathbuf to utf-8"))
            }) {
                Some(result) => Some(result?),
                None => None,
            }
        }
    };

    util::debug::log("Loading config file\n")?;
    let config = app::Config::load(config_location.as_deref()).unwrap_or_else(|err| {
        cross_eprintln!("Failed to load config: {}", err);
        cross::process::exit(1);
    });

    let ctx = app::Context {
        verbose: cli.verbose,
        config,
        config_location,
    };
    print_info!(ctx, 1, "Context generated!");

    print_info!(ctx, 1, "Fetching schema...");
    let schema = introspection::Response::fetch(&ctx.config).await?.schema();
    print_info!(ctx, 1, "Schema fetched!");

    if ctx.config.emit_schema {
        print_info!(ctx, 1, "Emitting schema json");
        // FIXME: Doesn't really make sense to serialize the schema again when we already recieved it in serial form...
        let schema_json =
            serde_json::to_string_pretty(&schema).expect("recieved valid schema json");
        cross::fs::write_to_file("schema.json", &schema_json)?;
    }

    print_info!(ctx, 1, "Generating typescript...");
    let ts = generate_typescript(cli, &ctx, schema).await?;

    cross::fs::write_to_file("generated.ts", &ts)?;

    Ok(())
}
