mod config;
mod introspection;
mod typescript;
mod util;

use std::fmt::{Display, Write as FmtWrite};
use std::io::Write;

use graphql_client::GraphQLQuery;
use typescript::WithIndexable;

use crate::introspection::IntrospectionResponse;
use crate::typescript::{TypeIndex, TypescriptableWithBuffer};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/introspection_schema.graphql",
    query_path = "src/graphql/introspection_query.graphql",
    response_derives = "Serialize",
    variable_derives = "Deserialize"
)]
struct IntrospectionQuery;

pub struct Buffer {
    pub imports: String,
    pub util_types: String,
    pub scalars: String,
    pub enums: String,
    pub objects: String,
    pub input_objects: String,
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
    if let Some(working_dir) = std::env::args().last() {
        std::env::set_current_dir(&working_dir)?;
    }

    let config = config::load().unwrap_or_else(|err| {
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
        selection_sets: String::new(),
        args: String::new(),
        queries: String::new(),
        mutations: String::new(),
        subscriptions: String::new(),
    };

    let body = IntrospectionQuery::build_query(introspection_query::Variables {});

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(config.no_ssl.unwrap_or(false))
        .build()?;

    let res = client.post(config.schema).json(&body).send().await?;

    let res: IntrospectionResponse = res.json().await?;

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

    let document = std::fs::read_to_string(config.document)?;
    let document = graphql_parser::parse_query::<&str>(&document)?;

    for def in &document.definitions {
        def.with_index(&type_index).as_typescript_on(&mut buffer)?;
    }

    for t in res.data.schema.types {
        t.as_typescript_on(&mut buffer)?;
    }

    write!(std::fs::File::create("generated.ts")?, "{}", buffer)?;

    Ok(())
}
