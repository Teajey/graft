use std::fmt::Write;

use convert_case::{Case, Casing};
use eyre::{eyre, Result};
use graphql_parser::query::{
    Definition, FragmentDefinition, FragmentSpread, InlineFragment, OperationDefinition,
    TypeCondition,
};

use super::{TypeIndex, Typescriptable, TypescriptableWithBuffer, WithIndex, WithIndexable};
use crate::{
    gen::Buffer,
    introspection::{Field, Type, TypeRef},
};

use graphql_parser::query::{Field as SelectedField, Selection, SelectionSet};

impl<'a> WithIndexable for Definition<'a, &'a str> {}

impl<'a, 'b, 'c, 'd> TypescriptableWithBuffer<'a>
    for WithIndex<'a, 'b, 'c, Definition<'d, &'d str>>
{
    fn as_typescript_on(&'a self, buffer: &mut Buffer) -> Result<()> {
        let definition = self.target;
        let type_index = self.type_index;

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
                        query.to_string(),
                        query
                            .name
                            .ok_or_else(|| eyre!("Encountered a query with no name."))?,
                        &mut buffer.queries,
                        "Query",
                        &query.variable_definitions,
                        &query.selection_set,
                        &type_index.query,
                    ),
                    OperationDefinition::Mutation(mutation) => (
                        mutation.to_string(),
                        mutation
                            .name
                            .ok_or_else(|| eyre!("Encountered a mutation with no name."))?,
                        &mut buffer.mutations,
                        "Mutation",
                        &mutation.variable_definitions,
                        &mutation.selection_set,
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
                        &subscription.variable_definitions,
                        &subscription.selection_set,
                        type_index.subscription.as_ref().ok_or_else(|| {
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

                let document_name = format!("{operation_name}{operation_type_name}Document");
                let args_name = format!("{operation_name}{operation_type_name}Args");
                let selection_set_name =
                    format!("{operation_name}{operation_type_name}SelectionSet");

                writeln!(
                    operation_buffer,
                    "export const {document_name} = parse(`{operation_ast}`) \
                  as TypedQueryDocumentNode<{selection_set_name}, {args_name}>;",
                )?;

                if variable_definitions.is_empty() {
                    writeln!(
                        buffer.args,
                        "export type {args_name} = Record<string, never>;"
                    )?;
                } else {
                    writeln!(buffer.args, "export type {args_name} = {{")?;
                    for def in variable_definitions {
                        let ts_type = TypeRef::try_from_arg(type_index, &def.var_type)?;
                        if let TypeRef::NonNull { .. } = ts_type {
                            writeln!(buffer.args, "  {}: {},", def.name, ts_type.as_typescript()?)?;
                        } else {
                            writeln!(
                                buffer.args,
                                "  {}?: {},",
                                def.name,
                                ts_type.as_typescript()?
                            )?;
                        }
                    }
                    writeln!(buffer.args, "}}")?;
                }

                let operation_fields = if let Type::Object { fields, .. } = operation_type {
                    fields
                } else {
                    return Err(eyre!("Top-level operation must be an object"));
                };
                write!(buffer.selection_sets, "export type {selection_set_name} = ",)?;
                recursively_typescriptify_selected_object_fields(
                    selection_set,
                    &mut buffer.selection_sets,
                    operation_fields,
                    type_index,
                )?;
                writeln!(buffer.selection_sets, ";")?;
            }
            Definition::Fragment(FragmentDefinition {
                position,
                name,
                type_condition,
                directives,
                selection_set,
            }) => {
                let TypeCondition::On(type_name) = type_condition;
                write!(
                    buffer.fragments,
                    "export type {}Fragment = ",
                    name.to_case(Case::Pascal)
                )?;
                recursively_typescriptify_selected_field(
                    selection_set,
                    &mut buffer.fragments,
                    &TypeRef::from(
                        type_index
                            .get(type_name)
                            .ok_or_else(|| eyre!("Type targetted by fragment not found"))?
                            .clone(),
                    ),
                    type_index,
                    &mut false,
                )?;
                writeln!(buffer.fragments, ";")?;
            }
        }

        Ok(())
    }
}

fn recursively_typescriptify_selected_object_fields<'a>(
    selection_set: &SelectionSet<'a, &'a str>,
    buffer: &mut String,
    selectable_fields: &[Field],
    type_index: &TypeIndex,
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
                    .ok_or_else(|| eyre!("Tried to select non-existent field at {position}"))?;
                let field_name = alias.unwrap_or(&selected_field.name);

                write!(buffer, "{}: ", field_name)?;

                recursively_typescriptify_selected_field(
                    selection_set,
                    buffer,
                    &selected_field.of_type,
                    type_index,
                    &mut true,
                )?;

                write!(buffer, ", ")?;
            }
            Selection::FragmentSpread(FragmentSpread {
                position,
                fragment_name,
                directives,
            }) => {
                fragment_strings.push(format!("{}Fragment", fragment_name.to_case(Case::Pascal)));
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
                        &TypeRef::from(
                            type_index
                                .get(type_name)
                                .ok_or_else(|| {
                                    eyre!("Type targetted by inline fragment not found")
                                })?
                                .clone(),
                        ),
                        type_index,
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

fn recursively_typescriptify_selected_field<'a>(
    selection_set: &SelectionSet<'a, &'a str>,
    buffer: &mut String,
    type_ref: &TypeRef,
    type_index: &TypeIndex,
    nullable: &mut bool,
) -> Result<()> {
    let selected_field_type = type_index.type_from_ref(type_ref);
    let mut local_buffer = String::new();

    match selected_field_type {
        Type::Object { fields, .. } => {
            recursively_typescriptify_selected_object_fields(
                selection_set,
                &mut local_buffer,
                &fields,
                type_index,
            )?;
        }
        Type::NonNull { of_type } => {
            *nullable = false;
            recursively_typescriptify_selected_field(
                selection_set,
                &mut local_buffer,
                &of_type,
                type_index,
                nullable,
            )?;
        }
        Type::List { of_type } => {
            recursively_typescriptify_selected_field(
                selection_set,
                &mut local_buffer,
                &of_type,
                type_index,
                nullable,
            )?;
            write!(local_buffer, "[]")?;
        }
        leaf_field_type => {
            write!(
                local_buffer,
                "{}",
                TypeRef::from(leaf_field_type).as_typescript_non_nullable()?
            )?;
        }
    };

    if *nullable {
        write!(buffer, "Nullable<{}>", local_buffer)?;
    } else {
        write!(buffer, "{}", local_buffer)?;
    }

    Ok(())
}
