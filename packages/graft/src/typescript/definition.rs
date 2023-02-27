use std::fmt::Write;

use convert_case::{Case, Casing};
use eyre::{eyre, Result};
use graphql_parser::query::{
    Definition, FragmentSpread, InlineFragment, OperationDefinition, TypeCondition,
};

use super::{TypescriptContext, Typescriptable, TypescriptableWithBuffer, WithContext};
use crate::{
    gen::Buffer,
    graphql::{
        query::{self as ac, Operation},
        schema::{Field, NamedType, Type, TypeRef, TypeRefContainer},
    },
};

use graphql_parser::query::{Field as SelectedField, Selection, SelectionSet};

impl<'a, 'b, 'c> TypescriptableWithBuffer for WithContext<'a, 'b, 'c, Definition<'_, String>> {
    fn as_typescript_on(&self, buffer: &mut Buffer) -> Result<()> {
        let definition = self.target;
        let ctx = self.ctx;

        match definition {
            Definition::Operation(operation_definition) => {
                let operation_bundle = match operation_definition {
                    OperationDefinition::SelectionSet(set) => {
                        return Err(eyre!(
                          "Top-level SelectionSets are not supported.\nThis selection set should be a query, mutation, or subscription:\n{}",
                          set
                      ));
                    }
                    OperationDefinition::Query(query) => (
                        Operation::from(OperationDefinition::Query(query.clone())),
                        query
                            .name
                            .as_ref()
                            .ok_or_else(|| eyre!("Encountered a query with no name."))?,
                        &mut buffer.queries,
                        "Query",
                        &query.variable_definitions,
                        &query.selection_set,
                        &ctx.index.query,
                    ),
                    OperationDefinition::Mutation(mutation) => (
                        Operation::from(OperationDefinition::Mutation(mutation.clone())),
                        mutation
                            .name
                            .as_ref()
                            .ok_or_else(|| eyre!("Encountered a mutation with no name."))?,
                        &mut buffer.mutations,
                        "Mutation",
                        &mutation.variable_definitions,
                        &mutation.selection_set,
                        ctx.index
                            .mutation
                            .as_ref()
                            .ok_or_else(|| eyre!("Mutation type does not exist in TypeIndex"))?,
                    ),
                    OperationDefinition::Subscription(subscription) => (
                        Operation::from(OperationDefinition::Subscription(subscription.clone())),
                        subscription
                            .name
                            .as_ref()
                            .ok_or_else(|| eyre!("Encountered a subscription with no name."))?,
                        &mut buffer.subscriptions,
                        "Subscription",
                        &subscription.variable_definitions,
                        &subscription.selection_set,
                        ctx.index.subscription.as_ref().ok_or_else(|| {
                            eyre!("Subscription type does not exist in TypeIndex")
                        })?,
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

                let document = ac::Document::new(vec![ac::Definition::Operation(operation_ast)]);

                let document_json = serde_json::to_string(&document)?;

                let operation_type_name = if ctx.options.documents_hide_operation_name {
                    ""
                } else {
                    operation_type_name
                };

                let document_name = format!("{operation_name}{operation_type_name}Document");
                let args_name = format!(
                    "{operation_name}{operation_type_name}{arguments_suffix}",
                    arguments_suffix = ctx.options.arguments_suffix
                );
                let selection_set_name = format!(
                    "{operation_name}{operation_type_name}{selection_set_suffix}",
                    selection_set_suffix = ctx.options.selection_set_suffix
                );

                writeln!(
                    operation_buffer,
                    "export const {document_name} = {document_json} as unknown as {document_type_name}<{selection_set_name}, {args_name}>;",
                    document_type_name = ctx.options.document_import.type_name()
                )?;

                if variable_definitions.is_empty() {
                    writeln!(
                        buffer.args,
                        "export type {args_name} = Record<string, never>;"
                    )?;
                } else {
                    writeln!(buffer.args, "export type {args_name} = {{")?;
                    for def in variable_definitions {
                        let ts_type = TypeRef::from(def.var_type.clone());
                        if ts_type.is_non_null() {
                            writeln!(
                                buffer.args,
                                "  {}: {},",
                                def.name,
                                ctx.with(&ts_type).as_typescript()?
                            )?;
                        } else {
                            writeln!(
                                buffer.args,
                                "  {}?: {},",
                                def.name,
                                ctx.with(&ts_type).as_typescript()?
                            )?;
                        }
                    }
                    writeln!(buffer.args, "}}")?;
                }

                let operation_fields = if let NamedType::Object { fields, .. } = operation_type {
                    fields
                } else {
                    return Err(eyre!("Top-level operation must be an object"));
                };
                write!(buffer.selection_sets, "export type {selection_set_name} = ",)?;
                recursively_typescriptify_selected_object_fields(
                    selection_set,
                    &mut buffer.selection_sets,
                    operation_fields,
                    ctx,
                )?;
                writeln!(buffer.selection_sets, ";")?;
            }
            Definition::Fragment(fragment) => {
                let definition = ac::Definition::from(Definition::Fragment(fragment.clone()));
                let document = ac::Document::new(vec![definition]);

                let document_json = serde_json::to_string(&document)?;

                writeln!(buffer.fragments, "export const {name}FragmentDocument = {document_json} as unknown as TypedQueryDocumentNode<{name}Fragment{selection_set_suffix}, unknown>", name = fragment.name.to_case(Case::Pascal), selection_set_suffix = ctx.options.selection_set_suffix)?;

                let TypeCondition::On(type_name) = &fragment.type_condition;
                write!(
                    buffer.selection_sets,
                    "export type {}Fragment{selection_set_suffix} = ",
                    fragment.name.to_case(Case::Pascal),
                    selection_set_suffix = ctx.options.selection_set_suffix
                )?;
                recursively_typescriptify_selected_field(
                    &fragment.selection_set,
                    &mut buffer.selection_sets,
                    &TypeRef::from(ctx.index.get(type_name).ok_or_else(|| {
                        eyre!(
                            "Type targetted by fragment at {} not found",
                            fragment.position
                        )
                    })?),
                    ctx,
                    &mut false,
                )?;
                writeln!(buffer.selection_sets, ";")?;
            }
        }

        Ok(())
    }
}

fn recursively_typescriptify_selected_object_fields(
    selection_set: &SelectionSet<'_, String>,
    buffer: &mut String,
    selectable_fields: &[Field],
    ctx: &TypescriptContext,
) -> Result<()> {
    let mut fragment_strings = Vec::<String>::new();
    write!(buffer, "{{ ")?;
    for selection in &selection_set.items {
        match selection {
            Selection::Field(SelectedField {
                position,
                alias,
                name,
                arguments: _,
                directives,
                selection_set,
            }) => {
                let selected_field = selectable_fields
                    .iter()
                    .find(|f| f.name == **name)
                    .ok_or_else(|| {
                        eyre!("Tried to select non-existent field '{name}' at {position}")
                    })?;
                let field_name = alias.as_ref().unwrap_or(&selected_field.name);

                write!(buffer, "{}: ", field_name)?;

                recursively_typescriptify_selected_field(
                    selection_set,
                    buffer,
                    &selected_field.of_type,
                    ctx,
                    &mut true,
                )?;

                write!(buffer, ", ")?;
            }
            Selection::FragmentSpread(FragmentSpread {
                position: _,
                fragment_name,
                directives,
            }) => {
                fragment_strings.push(format!(
                    "{}Fragment{selection_set_suffix}",
                    fragment_name.to_case(Case::Pascal),
                    selection_set_suffix = ctx.options.selection_set_suffix
                ));
            }
            Selection::InlineFragment(InlineFragment {
                position,
                type_condition,
                directives,
                selection_set,
            }) => {
                if let Some(TypeCondition::On(type_name)) = type_condition {
                    let mut fragment_buffer = String::new();
                    recursively_typescriptify_selected_field(
                        selection_set,
                        &mut fragment_buffer,
                        &TypeRef::from(ctx.index.get(type_name).ok_or_else(|| {
                            eyre!("Type targetted by inline fragment at {position} not found")
                        })?),
                        ctx,
                        &mut false,
                    )?;
                    fragment_strings.push(fragment_buffer);
                } else {
                    return Err(eyre!(
                        "Nameless/typeless inline fragments not (yet?) supported"
                    ));
                }
            }
        }
    }
    write!(buffer, "}}")?;

    if !fragment_strings.is_empty() {
        write!(buffer, " & {}", fragment_strings.join(" & "))?;
    }

    Ok(())
}

fn recursively_typescriptify_selected_field(
    selection_set: &SelectionSet<'_, String>,
    buffer: &mut String,
    type_ref: &TypeRef,
    ctx: &TypescriptContext,
    nullable: &mut bool,
) -> Result<()> {
    let selected_field_type = ctx.index.type_from_ref(type_ref.clone())?;
    let mut local_buffer = String::new();

    match selected_field_type {
        Type::Named(named_type) => match named_type {
            NamedType::Object { fields, .. } => {
                recursively_typescriptify_selected_object_fields(
                    selection_set,
                    &mut local_buffer,
                    &fields,
                    ctx,
                )?;
            }
            leaf_field_type => {
                write!(
                    local_buffer,
                    "{}",
                    ctx.with(&TypeRef::from(&leaf_field_type))
                        .as_typescript_non_nullable()?
                )?;
            }
        },
        Type::Container(contained) => match contained {
            TypeRefContainer::NonNull { of_type } => {
                *nullable = false;
                recursively_typescriptify_selected_field(
                    selection_set,
                    &mut local_buffer,
                    &of_type,
                    ctx,
                    nullable,
                )?;
            }
            TypeRefContainer::List { of_type } => {
                recursively_typescriptify_selected_field(
                    selection_set,
                    &mut local_buffer,
                    &of_type,
                    ctx,
                    nullable,
                )?;
                write!(local_buffer, "[]")?;
            }
        },
    };

    if *nullable {
        write!(buffer, "Nullable<{}>", local_buffer)?;
    } else {
        write!(buffer, "{}", local_buffer)?;
    }

    Ok(())
}
