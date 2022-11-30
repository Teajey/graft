use std::fmt::{Display, Write as FmtWrite};

use eyre::Result;
use graphql_parser::query::Document;

use crate::cli;
use crate::config;
use crate::cross;
use crate::introspection::Schema;
use crate::typescript::{TypeIndex, TypescriptableWithBuffer, WithIndexable};
use crate::util::path_with_possible_prefix;

pub struct Buffer {
    pub imports: String,
    pub util_types: String,
    pub scalars: String,
    pub enums: String,
    pub objects: String,
    pub input_objects: String,
    pub interfaces: String,
    pub unions: String,
    pub selection_sets: String,
    pub args: String,
    pub queries: String,
    pub mutations: String,
    pub subscriptions: String,
    pub fragments: String,
}

impl Display for Buffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buffer_buffer = String::new();

        writeln!(buffer_buffer, "{}", self.imports)?;
        writeln!(buffer_buffer, "// Utility types")?;
        writeln!(buffer_buffer, "{}", self.util_types)?;
        writeln!(buffer_buffer, "// Scalars")?;
        writeln!(buffer_buffer, "{}", self.scalars)?;
        writeln!(buffer_buffer, "// Enums")?;
        writeln!(buffer_buffer, "{}", self.enums)?;
        writeln!(buffer_buffer, "// Objects")?;
        writeln!(buffer_buffer, "{}", self.objects)?;
        writeln!(buffer_buffer, "// Input Objects")?;
        writeln!(buffer_buffer, "{}", self.input_objects)?;
        writeln!(buffer_buffer, "// Interfaces")?;
        writeln!(buffer_buffer, "{}", self.interfaces)?;
        writeln!(buffer_buffer, "// Unions")?;
        writeln!(buffer_buffer, "{}", self.unions)?;
        writeln!(buffer_buffer, "// Selection Sets")?;
        writeln!(buffer_buffer, "{}", self.selection_sets)?;
        writeln!(buffer_buffer, "// Args")?;
        writeln!(buffer_buffer, "{}", self.args)?;
        writeln!(buffer_buffer, "// Queries")?;
        writeln!(buffer_buffer, "{}", self.queries)?;
        writeln!(buffer_buffer, "// Mutations")?;
        writeln!(buffer_buffer, "{}", self.mutations)?;
        writeln!(buffer_buffer, "// Subscriptions")?;
        writeln!(buffer_buffer, "{}", self.subscriptions)?;
        writeln!(buffer_buffer, "// Fragments")?;
        write!(buffer_buffer, "{}", self.fragments)?;

        write!(f, "{}", buffer_buffer)
    }
}

pub async fn generate_typescript_with_document<'a>(
    cli: cli::Base,
    schema: Schema,
    document: Option<Document<'a, &'a str>>,
) -> Result<String> {
    let mut buffer = Buffer {
        imports: String::new(),
        util_types: String::new(),
        scalars: String::new(),
        enums: String::new(),
        objects: String::new(),
        input_objects: String::new(),
        interfaces: String::new(),
        unions: String::new(),
        selection_sets: String::new(),
        args: String::new(),
        queries: String::new(),
        mutations: String::new(),
        subscriptions: String::new(),
        fragments: String::new(),
    };

    let type_index = TypeIndex::try_new(&schema)?;

    writeln!(
        buffer.imports,
        r#"import {{ parse, TypedQueryDocumentNode }} from "graphql";"#
    )?;

    writeln!(buffer.util_types, "export type Nullable<T> = T | null;")?;
    writeln!(
        buffer.util_types,
        "export type NewType<T, U> = T & {{ readonly __newtype: U }};"
    )?;

    if let Some(document) = document {
        for def in &document.definitions {
            def.with_index(&type_index).as_typescript_on(&mut buffer)?;
        }
    }

    for t in schema.types {
        t.as_typescript_on(&mut buffer)?;
    }

    Ok(buffer.to_string())
}

pub async fn generate_typescript(
    cli: cli::Base,
    config: config::AppConfig,
    schema: Schema,
) -> Result<String> {
    let Some(document_path) = &config.document_path else {
        return generate_typescript_with_document(cli, schema, None)
        .await;
    };

    let document_path = path_with_possible_prefix(cli.config_location.as_deref(), document_path);

    let document_str = cross::fs::read_to_string(document_path)?;

    let document = graphql_parser::parse_query::<&str>(&document_str)?;

    generate_typescript_with_document(cli, schema, Some(document)).await
}

// Native test only for now...
#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use eyre::Result;

    use crate::introspection;
    use crate::{cli, config, gen::generate_typescript};

    #[tokio::test]
    async fn typescript_output_matches_snapshot() -> Result<()> {
        let cli = cli::Base {
            working_directory: None,
            config_location: None,
        };

        let config = config::RawAppConfig {
            schema: "https://swapi-graphql.netlify.app/.netlify/functions/index".to_owned(),
            no_ssl: None,
            document: Some(PathBuf::from("../testing/document.graphql")),
            emit_schema: None,
        };

        let config = config::AppConfig::try_from(config)?;

        let schema = introspection::Response::fetch(&config).await?.schema();

        let typescript = generate_typescript(cli, config, schema).await?;

        insta::assert_snapshot!(typescript);

        Ok(())
    }
}
