mod cli;
mod config;
mod introspection;
mod typescript;
mod util;

use std::fmt::{Display, Write as FmtWrite};
use std::io::Write;

use clap::Parser;

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
    pub selection_sets: String,
    pub args: String,
    pub queries: String,
    pub mutations: String,
    pub subscriptions: String,
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
        writeln!(buffer_buffer, "// Selection Sets")?;
        writeln!(buffer_buffer, "{}", self.selection_sets)?;
        writeln!(buffer_buffer, "// Args")?;
        writeln!(buffer_buffer, "{}", self.args)?;
        writeln!(buffer_buffer, "// Queries")?;
        writeln!(buffer_buffer, "{}", self.queries)?;
        writeln!(buffer_buffer, "// Mutations")?;
        writeln!(buffer_buffer, "{}", self.mutations)?;
        writeln!(buffer_buffer, "// Subscriptions")?;
        write!(buffer_buffer, "{}", self.subscriptions)?;

        write!(f, "{}", buffer_buffer)
    }
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let cli = cli::Base::parse();

    if let Some(working_dir) = cli.working_directory {
        std::env::set_current_dir(&working_dir)?;
    }

    let config = config::load(cli.config_location.as_deref()).unwrap_or_else(|err| {
        eprintln!("Failed to load config: {}", err);
        std::process::exit(1);
    });

    let mut buffer = Buffer {
        imports: String::new(),
        util_types: String::new(),
        scalars: String::new(),
        enums: String::new(),
        objects: String::new(),
        input_objects: String::new(),
        interfaces: String::new(),
        selection_sets: String::new(),
        args: String::new(),
        queries: String::new(),
        mutations: String::new(),
        subscriptions: String::new(),
    };

    let res = introspection::Response::fetch(&config).await?;

    let type_index = TypeIndex::try_new(&res.data.schema)?;

    writeln!(
        buffer.imports,
        r#"import {{ parse, TypedQueryDocumentNode }} from "graphql";"#
    )?;

    writeln!(buffer.util_types, "export type Nullable<T> = T | null;")?;
    writeln!(
        buffer.util_types,
        "export type NewType<T, U> = T & {{ readonly __newtype: U }};"
    )?;

    let document_path = path_with_possible_prefix(cli.config_location.as_deref(), &config.document);

    if document_path.exists() {
        let document = std::fs::read_to_string(document_path)?;
        if !document.is_empty() {
            let document = graphql_parser::parse_query::<&str>(&document)?;

            for def in &document.definitions {
                def.with_index(&type_index).as_typescript_on(&mut buffer)?;
            }
        }
    }

    for t in res.data.schema.types {
        t.as_typescript_on(&mut buffer)?;
    }

    write!(std::fs::File::create("generated.ts")?, "{}", buffer)?;

    Ok(())
}
