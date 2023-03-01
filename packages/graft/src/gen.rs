use std::fmt::{Display, Write as FmtWrite};

use eyre::Result;
use graphql_parser::query::Document;

use crate::app;
use crate::app::config::{TypescriptOptions, DocumentPaths};
use crate::debug_log;
use crate::graphql::schema::Schema;
use crate::typescript::{self, TypeIndex, TypescriptableWithBuffer};

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

        write!(f, "{buffer_buffer}")
    }
}

pub fn generate_typescript_with_document(
    options: TypescriptOptions,
    schema: &Schema,
    document: Option<Document<'_, String>>,
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

    let index = TypeIndex::try_new(schema)?;

    let ctx = typescript::Context { index, options };

    writeln!(
        buffer.imports,
        r#"import type {{ {type_name} }} from "{package}";"#,
        type_name = ctx.options.document_import.type_name(),
        package = ctx.options.document_import.package(),
    )?;

    writeln!(buffer.util_types, "export type Nullable<T> = T | null;")?;
    writeln!(
        buffer.util_types,
        "export type NewType<T, U> = T & {{ readonly __newtype: U }};"
    )?;

    if let Some(document) = document {
        for def in document.definitions {
            ctx.with(&def).as_typescript_on(&mut buffer)?;
        }
    }

    for t in &schema.types {
        ctx.with(t).as_typescript_on(&mut buffer)?;
    }

    Ok(buffer.to_string())
}

pub fn generate_typescript(
    ctx: &app::Context,
    options: TypescriptOptions,
    document_paths: Option<DocumentPaths>,
    schema: &Schema,
) -> Result<String> {
    debug_log!("current dir files: {:?}", std::fs::read_dir("./"));

    let Some(document_paths) = document_paths else {
        return generate_typescript_with_document(options, schema, None);
    };
    
    let Some(full_document_string) = document_paths.resolve_to_full_document_string(ctx.config_location.as_deref())? else {
        return generate_typescript_with_document(options, schema, None);
    };

    debug_log!("AST: {}", full_document_string);


    let document = graphql_parser::parse_query::<String>(&full_document_string)?;
    debug_log!("Parsed document!");

    generate_typescript_with_document(options, schema, Some(document))
}

// Native test only for now...
#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod test {
    use eyre::Result;

    use crate::{
        app::{
            self,
            config::{DocumentImport, TypescriptOptions, DocumentPaths},
        },
        gen::generate_typescript,
        graphql::schema::Schema,
        introspection,
    };

    async fn context_and_schema() -> Result<(app::Context, Schema)> {
        let ctx = app::Context {
            verbose: 0,
            config_location: None,
        };

        let schema = introspection::Response::fetch(
            &ctx,
            "https://swapi-graphql.netlify.app/.netlify/functions/index",
            false,
        )
        .await?
        .schema()?;

        Ok((ctx, schema))
    }

    #[tokio::test]
    async fn standard_typescript() -> Result<()> {
        let (ctx, schema) = context_and_schema().await?;

        let typescript = generate_typescript(
            &ctx,
            TypescriptOptions::default(),
            Some(DocumentPaths::from([
                "../../examples/app/fragments.graphql",
                "../../examples/app/queries.graphql",
            ])),
            &schema,
        )?;

        insta::assert_snapshot!(typescript);

        Ok(())
    }

    #[tokio::test]
    async fn codegen_like_typescript() -> Result<()> {
        let (ctx, schema) = context_and_schema().await?;

        let typescript = generate_typescript(
            &ctx,
            TypescriptOptions {
                document_import: DocumentImport::default(),
                scalar_newtypes: None,
                documents_hide_operation_name: true,
                selection_set_suffix: String::new(),
                arguments_suffix: "Variables".to_owned(),
            },
            Some(DocumentPaths::from([
                "../../examples/app/fragments.graphql",
                "../../examples/app/queries.graphql",
            ])),
            &schema,
        )?;

        insta::assert_snapshot!(typescript);

        Ok(())
    }
}
