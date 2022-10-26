mod config;
mod introspection_response;

use std::fmt::Write as FmtWrite;
use std::io::Write;

use convert_case::{Case, Casing};
use eyre::Result;
use graphql_client::GraphQLQuery;

use introspection_response::{IntrospectionResponse, Type};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/introspection_schema.graphql",
    query_path = "src/graphql/introspection_query.graphql",
    response_derives = "Serialize",
    variable_derives = "Deserialize"
)]
struct IntrospectionQuery;

fn possibly_write_description<W: FmtWrite>(out: &mut W, description: Option<String>) -> Result<()> {
    if let Some(description) = description {
        if description.contains('\n') {
            writeln!(out, "/**\n * {}\n */", description.replace('\n', "\n * "))?;
        } else {
            writeln!(out, "/** {} */", description)?;
        }
    };

    Ok(())
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let config = config::load().unwrap_or_else(|err| {
        eprintln!("Failed to load config: {}", err);
        std::process::exit(1);
    });

    let mut out = std::fs::File::create("generated.ts")?;
    let schema_out = std::fs::File::create("schema.json")?;
    let mut util_types = String::new();
    let mut scalars = String::new();
    let mut enums = String::new();
    let mut objects = String::new();

    let body = IntrospectionQuery::build_query(introspection_query::Variables {});

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(config.no_ssl.unwrap_or(false))
        .build()?;

    let res = client.post(config.schema).json(&body).send().await?;

    let res: IntrospectionResponse = res.json().await?;

    serde_json::to_writer_pretty(schema_out, &res)?;

    writeln!(util_types, "export type Nullable<T> = T | null;")?;
    writeln!(
        util_types,
        "export type NewType<T, U> = T & {{ readonly __newtype: U }};"
    )?;

    for t in res.data.schema.types {
        match t {
            Type::Scalar { name, description } => {
                possibly_write_description(&mut scalars, description)?;
                let scalar_type = match name.as_str() {
                    "ID" => r#"NewType<string, "ID">"#,
                    "String" => "string",
                    "Int" => "number",
                    "Float" => "number",
                    "Boolean" => "boolean",
                    _ => "unknown",
                };
                writeln!(scalars, "type {name}Scalar = {scalar_type};")?;
            }
            Type::Enum {
                name,
                description,
                enum_values,
            } => {
                if name.starts_with('_') {
                    continue;
                }
                possibly_write_description(&mut enums, description)?;
                writeln!(enums, "enum {name} {{")?;
                for v in enum_values {
                    if let Some(description) = v.description {
                        writeln!(enums, "/**\n * {}\n */", description.replace('\n', "\n * "))?;
                    }
                    writeln!(
                        enums,
                        "  {} = \"{}\",",
                        v.name.to_case(Case::Pascal),
                        v.name
                    )?;
                }
                writeln!(enums, "}}")?;
            }
            Type::Object {
                name,
                description,
                fields,
                interfaces,
            } => {
                if name.starts_with('_') {
                    continue;
                }
                possibly_write_description(&mut objects, description)?;
                writeln!(objects, "type {name} = {{")?;
                for f in fields {
                    possibly_write_description(&mut objects, f.description)?;
                    writeln!(objects, "  {}: {},", f.name, f.of_type)?;
                }
                writeln!(objects, "}}")?;
            }
            _ => (),
        }
    }

    writeln!(out, "// Utility types")?;
    writeln!(out, "{}", util_types)?;
    writeln!(out, "// Scalars")?;
    writeln!(out, "{}", scalars)?;
    writeln!(out, "// Enums")?;
    writeln!(out, "{}", enums)?;
    writeln!(out, "// Object Types")?;
    writeln!(out, "{}", objects)?;

    Ok(())
}
