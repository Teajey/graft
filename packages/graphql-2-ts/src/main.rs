mod config;
mod introspection_response;
mod transform;
mod util;

use std::fmt::Write as FmtWrite;
use std::io::Write;

use convert_case::{Case, Casing};
use eyre::{eyre, Result};
use graphql_client::GraphQLQuery;
use graphql_parser::query::{Definition, OperationDefinition};
use transform::recursively_typescriptify_selected_object_fields;

use crate::introspection_response::{IntrospectionResponse, Type, TypeRef};
use crate::transform::try_type_ref_from_arg;
use crate::util::TypeIndex;

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
    if let Some(working_dir) = std::env::args().nth(1) {
        std::env::set_current_dir(working_dir)?;
    }

    let config = config::load().unwrap_or_else(|err| {
        eprintln!("Failed to load config: {}", err);
        std::process::exit(1);
    });

    let mut out = std::fs::File::create("generated.ts")?;
    let schema_out = std::fs::File::create("schema.json")?;

    let mut imports = String::new();
    let mut util_types = String::new();
    let mut scalars = String::new();
    let mut enums = String::new();
    let mut objects = String::new();
    let mut input_objects = String::new();
    let mut selection_sets = String::new();
    let mut args = String::new();
    let mut queries = String::new();
    let mut mutations = String::new();
    let mut subscriptions = String::new();

    let body = IntrospectionQuery::build_query(introspection_query::Variables {});

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(config.no_ssl.unwrap_or(false))
        .build()?;

    let res = client.post(config.schema).json(&body).send().await?;

    let res: IntrospectionResponse = res.json().await?;

    let type_index = TypeIndex::try_new(&res.data.schema)?;

    serde_json::to_writer_pretty(schema_out, &res)?;

    writeln!(
        imports,
        r#"import {{ parse, TypedQueryDocumentNode }} from "graphql";"#
    )?;

    writeln!(util_types, "export type Nullable<T> = T | null;")?;
    writeln!(
        util_types,
        "export type NewType<T, U> = T & {{ readonly __newtype: U }};"
    )?;

    let document = std::fs::read_to_string(config.document)?;
    let document = graphql_parser::parse_query::<&str>(&document)?;

    for def in document.definitions {
        if let Definition::Operation(operation_definition) = def {
            let operation_bundle = match operation_definition {
                OperationDefinition::SelectionSet(set) => {
                    return Err(eyre!(
                        "Top-level SelectionSets are not supported.\nThis selection set should be a query, mutation, or subscription:\n{}",
                        set
                    ))
                }
                OperationDefinition::Query(query) => (
                    query.to_string(),
                    query
                        .name
                        .ok_or_else(|| eyre!("Encountered a query with no name."))?,
                    &mut queries,
                    "Query",
                    query.variable_definitions,
                    query.selection_set,
                    &type_index.query
                ),
                OperationDefinition::Mutation(mutation) => (
                    mutation.to_string(),
                    mutation
                        .name
                        .ok_or_else(|| eyre!("Encountered a mutation with no name."))?,
                    &mut mutations,
                    "Mutation",
                    mutation.variable_definitions,
                    mutation.selection_set,
                    type_index.mutation.as_ref().ok_or_else(|| eyre!("Mutation type does not exist in TypeIndex"))?
                ),
                OperationDefinition::Subscription(subscription) => (
                    subscription.to_string(),
                    subscription
                        .name
                        .ok_or_else(|| eyre!("Encountered a subscription with no name."))?,
                    &mut subscriptions,
                    "Subscription",
                    subscription.variable_definitions,
                    subscription.selection_set,
                    type_index.subscription.as_ref().ok_or_else(|| eyre!("Subscription type does not exist in TypeIndex"))?
                ),
            };
            let (
                operation_ast,
                operation_name,
                string_buffer,
                operation_type_name,
                variable_definitions,
                selection_set,
                operation_type,
            ) = operation_bundle;
            let operation_name = operation_name.to_case(Case::Pascal);

            let document_name = format!("{operation_name}{operation_type_name}Document");
            let args_name = format!("{operation_name}{operation_type_name}Args");
            let selection_set_name = format!("{operation_name}{operation_type_name}SelectionSet");

            writeln!(
                string_buffer,
                "const {document_name} = parse(`{operation_ast}`) \
                as TypedQueryDocumentNode<{selection_set_name}, {args_name}>;",
            )?;

            writeln!(args, "type {args_name} = {{")?;
            for def in variable_definitions {
                let ts_type = try_type_ref_from_arg(&type_index, &def.var_type)?;
                if let TypeRef::NonNull { .. } = ts_type {
                    writeln!(args, "  {}: {},", def.name, ts_type)?;
                } else {
                    writeln!(args, "  {}?: {},", def.name, ts_type)?;
                }
            }
            writeln!(args, "}}")?;

            let operation_fields = if let Type::Object { fields, .. } = operation_type {
                fields
            } else {
                return Err(eyre!("Top-level operation must be an object"));
            };
            write!(selection_sets, "type {selection_set_name} = {{ ",)?;
            recursively_typescriptify_selected_object_fields(
                selection_set,
                &mut selection_sets,
                operation_fields,
                &type_index,
            )?;
            writeln!(selection_sets, "}};")?;
        }
    }

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
                    possibly_write_description(&mut enums, v.description)?;
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
            Type::InputObject {
                name,
                description,
                input_fields,
            } => {
                if name.starts_with('_') {
                    continue;
                }
                possibly_write_description(&mut objects, description)?;
                writeln!(input_objects, "type {name} = {{")?;
                for f in input_fields {
                    possibly_write_description(&mut input_objects, f.description)?;
                    if let TypeRef::NonNull { .. } = f.of_type {
                        writeln!(input_objects, "  {}: {},", f.name, f.of_type)?;
                    } else {
                        writeln!(input_objects, "  {}?: {},", f.name, f.of_type)?;
                    }
                }
                writeln!(input_objects, "}}")?;
            }
            _ => (),
        }
    }

    writeln!(out, "{}", imports)?;
    writeln!(out, "// Utility types")?;
    writeln!(out, "{}", util_types)?;
    writeln!(out, "// Scalars")?;
    writeln!(out, "{}", scalars)?;
    writeln!(out, "// Enums")?;
    writeln!(out, "{}", enums)?;
    writeln!(out, "// Objects")?;
    writeln!(out, "{}", objects)?;
    writeln!(out, "// Input Objects")?;
    writeln!(out, "{}", input_objects)?;
    writeln!(out, "// Selection Sets")?;
    writeln!(out, "{}", selection_sets)?;
    writeln!(out, "// Args")?;
    writeln!(out, "{}", args)?;
    writeln!(out, "// Queries")?;
    writeln!(out, "{}", queries)?;
    writeln!(out, "// Mutations")?;
    writeln!(out, "{}", mutations)?;
    writeln!(out, "// Subscriptions")?;
    writeln!(out, "{}", subscriptions)?;

    Ok(())
}
