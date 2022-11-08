use clap::Parser;
use wasm_bindgen::prelude::*;

use crate::gen::generate_typescript_with_document;
use crate::util::path_with_possible_prefix;
use crate::{cli, config, node, node_stdout};

fn read_to_string(path: &str) -> Result<String, JsValue> {
    let file_str: Vec<u8> = serde_wasm_bindgen::from_value(node::read_file(path)?.into())?;
    String::from_utf8(file_str)
        .map_err(|err| JsValue::from_str(&format!("file_str was not unicode: {err}")))
}

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

    let document_str = read_to_string(document_path)?;

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

    let cli_result = cli::Base::try_parse_from(
        argv.iter()
            .map(|v| v.as_string().ok_or(v))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|arg| JsValue::from_str(&format!("Failed to stringify argument: {arg:?}")))?,
    );

    match cli_result {
        Err(err) => {
            node_stdout!("{err}");
        }
        Ok(cli) => {
            let path = path_with_possible_prefix(cli.config_location.as_deref(), ".accord.yml");
            let path = path
                .as_os_str()
                .to_str()
                .ok_or_else(|| JsValue::from_str("Couldn't stringify config path"))?;

            let config_str = read_to_string(path)?;

            let config: config::AppConfig = serde_yaml::from_str(&config_str)
                .map_err(|err| JsValue::from_str(&format!("Failed to parse config_str: {err}")))?;

            let ts = generate_typescript(cli, config).await?;

            node::write_file("generated.ts", &ts)?;
        }
    };

    Ok(())
}
