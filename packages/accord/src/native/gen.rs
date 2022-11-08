use eyre::Result;

use crate::gen::generate_typescript_with_document;
use crate::util::path_with_possible_prefix;
use crate::{cli, config};

pub async fn generate_typescript(cli: cli::Base, config: config::AppConfig) -> Result<String> {
    let ts = if let Some(document_path) = &config.document_path {
        let document_path =
            path_with_possible_prefix(cli.config_location.as_deref(), document_path);

        if !document_path.exists() {
            generate_typescript_with_document(cli, config, None).await?
        } else {
            let document_str = std::fs::read_to_string(document_path)?;
            if document_str.is_empty() {
                generate_typescript_with_document(cli, config, None).await?
            } else {
                let document = graphql_parser::parse_query::<&str>(&document_str)?;
                generate_typescript_with_document(cli, config, Some(document)).await?
            }
        }
    } else {
        generate_typescript_with_document(cli, config, None).await?
    };

    Ok(ts)
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use eyre::Result;
    use url::Url;

    use crate::{cli, config, native::gen::generate_typescript};

    #[tokio::test]
    async fn typescript_output_matches_snapshot() -> Result<()> {
        let cli = cli::Base {
            working_directory: None,
            config_location: None,
        };

        let config = config::AppConfig {
            schema: Url::from_str("https://swapi-graphql.netlify.app/.netlify/functions/index")?,
            no_ssl: None,
            document_path: Some("../testing/document.graphql".to_owned()),
        };

        let typescript = generate_typescript(cli, config).await?;

        insta::assert_snapshot!(typescript);

        Ok(())
    }
}
