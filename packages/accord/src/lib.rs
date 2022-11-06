mod cli;
mod common;
mod introspection;
mod node;
mod typescript;
mod util;

use clap::Parser;
use wasm_bindgen::prelude::*;

use crate::common::{config, gen::generate_typescript_with_document};
use crate::util::path_with_possible_prefix;

async fn generate_typescript(cli: cli::Base, config: config::AppConfig) -> Result<String, JsValue> {
    let Some(document_path) = &config.document_path else {
        return generate_typescript_with_document(cli, config, None)
        .await
        .map_err(|err| JsValue::from_str(&err.to_string()));
    };

    let document_path = path_with_possible_prefix(cli.config_location.as_deref(), document_path);
    let document_path = document_path
        .as_os_str()
        .to_str()
        .ok_or_else(|| JsValue::from_str("Couldn't stringify document_path"))?;

    let document_str = node::read_file(document_path)?
        .as_string()
        .ok_or_else(|| JsValue::from_str("document file Buffer wasn't a string"))?;

    if document_str.is_empty() {
        return generate_typescript_with_document(cli, config, None)
            .await
            .map_err(|err| JsValue::from_str(&err.to_string()));
    }

    let document = graphql_parser::parse_query::<&str>(&document_str).map_err(|err| {
        JsValue::from_str(&format!("Couldn't parse document as GraphQL AST: {err}"))
    })?;

    generate_typescript_with_document(cli, config, Some(document))
        .await
        .map_err(|err| JsValue::from_str(&err.to_string()))
}

#[wasm_bindgen(start)]
pub async fn node_main() -> Result<(), JsValue> {
    let argv = node::argv();

    let cli = cli::Base::try_parse_from(
        argv.iter()
            .map(|v| v.as_string().ok_or(v))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|arg| JsValue::from_str(&format!("Failed to stringify argument: {arg:?}")))?,
    )
    .map_err(|err| JsValue::from_str(&format!("Failed to parse argv_str: {err}")))?;

    let config_str = node::read_file(".accord.yml")?
        .as_string()
        .ok_or_else(|| JsValue::from_str(".accord.yml Buffer wasn't a string"))?;

    let config: config::AppConfig = serde_yaml::from_str(&config_str)
        .map_err(|err| JsValue::from_str(&format!("Failed to parse config_str: {err}")))?;

    let ts = generate_typescript(cli, config).await?;

    node::write_file("generated.ts", &ts)?;

    Ok(())
}
