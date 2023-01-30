use clap::Parser;
use eyre::Result;
use graphql_parser::schema::Document;

// use crate::gen::generate_typescript;
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

    util::debug::log("Loading config file\n")?;
    let config = app::Config::load(cli.config_location.as_deref()).unwrap_or_else(|err| {
        cross_eprintln!("Failed to load config: {}", err);
        cross::process::exit(1);
    });

    let ctx = app::Context {
        verbose: cli.verbose,
        config_location: cli.config_location.clone(),
    };
    print_info!(ctx, 1, "Context generated!");

    for (name, plans) in config.generates {
        if let Some(schema_gen_plan) = plans.schema_gen_plan {
            print_info!(ctx, 1, "Fetching schema for {name}...");
            let schema = introspection::Response::fetch(
                &ctx,
                schema_gen_plan.url.0.as_str(),
                schema_gen_plan.no_ssl,
            )
            .await?
            .schema();
            print_info!(ctx, 1, "Schema fetched!");

            if let Some(json_path) = schema_gen_plan.out.json_path {
                print_info!(ctx, 1, "Emitting schema json");
                let schema_json = serde_json::to_string_pretty(&schema)?;
                cross::fs::write_to_file(json_path, &schema_json)?;
            }
            if let Some(ast_path) = schema_gen_plan.out.ast_path {
                let schema_graphql = format!("{}", Document::from(&schema));
                cross::fs::write_to_file(ast_path, &schema_graphql)?;
            }
        }
        // if let Some(typescript_gen_plan) = plans.typescript_gen_plan {
        //     print_info!(ctx, 1, "Generating typescript...");
        //     let ts = generate_typescript(&ctx, typescript_gen_plan, &schema).await?;

        //     cross::fs::write_to_file("generated.ts", &ts)?;
        // }
    }

    Ok(())
}
