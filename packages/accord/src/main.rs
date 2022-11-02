mod config;
mod introspection_response;
mod transform;
mod util;

use std::fmt::{Display, Write as FmtWrite};
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

struct Buffer {
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
    if let Some(working_dir) = std::env::args().nth(1) {
        std::env::set_current_dir(working_dir)?;
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

    for def in document.definitions {
        if let Definition::Operation(operation_definition) = def {
            let operation_bundle = match operation_definition {
                OperationDefinition::SelectionSet(set) => {
                    return Err(eyre!(
                        "Top-level SelectionSets are not supported.\nThis selection set should be a query, mutation, or subscription:\n{}",
                        set
                    ));
                }
                OperationDefinition::Query(query) => (
                    query.to_string(),
                    query
                        .name
                        .ok_or_else(|| eyre!("Encountered a query with no name."))?,
                    &mut buffer.queries,
                    "Query",
                    query.variable_definitions,
                    query.selection_set,
                    &type_index.query,
                ),
                OperationDefinition::Mutation(mutation) => (
                    mutation.to_string(),
                    mutation
                        .name
                        .ok_or_else(|| eyre!("Encountered a mutation with no name."))?,
                    &mut buffer.mutations,
                    "Mutation",
                    mutation.variable_definitions,
                    mutation.selection_set,
                    type_index
                        .mutation
                        .as_ref()
                        .ok_or_else(|| eyre!("Mutation type does not exist in TypeIndex"))?,
                ),
                OperationDefinition::Subscription(subscription) => (
                    subscription.to_string(),
                    subscription
                        .name
                        .ok_or_else(|| eyre!("Encountered a subscription with no name."))?,
                    &mut buffer.subscriptions,
                    "Subscription",
                    subscription.variable_definitions,
                    subscription.selection_set,
                    type_index
                        .subscription
                        .as_ref()
                        .ok_or_else(|| eyre!("Subscription type does not exist in TypeIndex"))?,
                ),
            };
            let (
                operation_ast,
                operation_name,
                operation_buffer,
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
                operation_buffer,
                "export const {document_name} = parse(`{operation_ast}`) \
                as TypedQueryDocumentNode<{selection_set_name}, {args_name}>;",
            )?;

            writeln!(buffer.args, "export type {args_name} = {{")?;
            for def in variable_definitions {
                let ts_type = try_type_ref_from_arg(&type_index, &def.var_type)?;
                if let TypeRef::NonNull { .. } = ts_type {
                    writeln!(buffer.args, "  {}: {},", def.name, ts_type)?;
                } else {
                    writeln!(buffer.args, "  {}?: {},", def.name, ts_type)?;
                }
            }
            writeln!(buffer.args, "}}")?;

            let operation_fields = if let Type::Object { fields, .. } = operation_type {
                fields
            } else {
                return Err(eyre!("Top-level operation must be an object"));
            };
            write!(
                buffer.selection_sets,
                "export type {selection_set_name} = {{ ",
            )?;
            recursively_typescriptify_selected_object_fields(
                selection_set,
                &mut buffer.selection_sets,
                operation_fields,
                &type_index,
            )?;
            writeln!(buffer.selection_sets, "}};")?;
        }
    }

    for t in res.data.schema.types {
        match t {
            Type::Scalar { name, description } => {
                possibly_write_description(&mut buffer.scalars, description)?;
                let scalar_type = match name.as_str() {
                    "ID" => r#"NewType<string, "ID">"#,
                    "String" => "string",
                    "Int" => "number",
                    "Float" => "number",
                    "Boolean" => "boolean",
                    _ => "unknown",
                };
                writeln!(buffer.scalars, "export type {name}Scalar = {scalar_type};")?;
            }
            Type::Enum {
                name,
                description,
                enum_values,
            } => {
                if name.starts_with('_') {
                    continue;
                }
                possibly_write_description(&mut buffer.enums, description)?;
                writeln!(buffer.enums, "export enum {name} {{")?;
                for v in enum_values {
                    possibly_write_description(&mut buffer.enums, v.description)?;
                    writeln!(
                        buffer.enums,
                        "  {} = \"{}\",",
                        v.name.to_case(Case::Pascal),
                        v.name
                    )?;
                }
                writeln!(buffer.enums, "}}")?;
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
                possibly_write_description(&mut buffer.objects, description)?;
                writeln!(buffer.objects, "export type {name} = {{")?;
                for f in fields {
                    possibly_write_description(&mut buffer.objects, f.description)?;
                    writeln!(buffer.objects, "  {}: {},", f.name, f.of_type)?;
                }
                writeln!(buffer.objects, "}}")?;
            }
            Type::InputObject {
                name,
                description,
                input_fields,
            } => {
                if name.starts_with('_') {
                    continue;
                }
                possibly_write_description(&mut buffer.objects, description)?;
                writeln!(buffer.input_objects, "export type {name} = {{")?;
                for f in input_fields {
                    possibly_write_description(&mut buffer.input_objects, f.description)?;
                    if let TypeRef::NonNull { .. } = f.of_type {
                        writeln!(buffer.input_objects, "  {}: {},", f.name, f.of_type)?;
                    } else {
                        writeln!(buffer.input_objects, "  {}?: {},", f.name, f.of_type)?;
                    }
                }
                writeln!(buffer.input_objects, "}}")?;
            }
            Type::Union {
                name,
                description,
                possible_types,
            } => todo!(),
            Type::Interface {
                name,
                description,
                fields,
                possible_types,
            } => todo!(),
            Type::List { .. } => return Err(eyre!("Top-level lists not supported.")),
            Type::NonNull { .. } => return Err(eyre!("Top-level non-nulls not supported.")),
        }
    }

    write!(std::fs::File::create("generated.ts")?, "{}", buffer)?;

    Ok(())
}
