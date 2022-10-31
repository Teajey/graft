use std::fmt::Write;

use eyre::{eyre, Result};
use graphql_parser::query::{Field as SelectedField, Selection, SelectionSet};

use crate::introspection_response::{Field, Type, TypeRef};
use crate::util::{Arg, MaybeNamed, TypeIndex};

pub trait TypescriptableGraphQLType {
    const NULL_WRAPPER_TYPE: &'static str = "Nullable";

    fn recursive_typescriptify(
        &self,
        string: Option<String>,
        wrap_me: bool,
    ) -> Result<(String, bool)>;

    fn as_typescript(&self) -> Result<String> {
        Ok(self.recursive_typescriptify(None, false)?.0)
    }

    fn wrap_if_nullable(string: String, wrap_me: bool) -> (String, bool) {
        let string = if wrap_me {
            format!("{}<{string}>", Self::NULL_WRAPPER_TYPE)
        } else {
            string
        };

        (string, wrap_me)
    }
}

pub fn try_type_ref_from_arg<'a>(type_index: &TypeIndex, arg: &Arg<'a>) -> Result<TypeRef> {
    match arg {
        Arg::NamedType(var_type_name) => type_index
            .get(var_type_name)
            .ok_or_else(|| {
                eyre!(
                    "Found a query argument type not defined in the schema: {}",
                    var_type_name
                )
            })
            .map(|t| t.clone().into()),
        Arg::NonNullType(var_type) => {
            let type_ref = try_type_ref_from_arg(type_index, var_type)?;
            Ok(TypeRef::NonNull {
                of_type: Box::new(type_ref),
            })
        }
        Arg::ListType(var_type) => {
            let type_ref = try_type_ref_from_arg(type_index, var_type)?;
            Ok(TypeRef::List {
                of_type: Box::new(type_ref),
            })
        }
    }
}

impl TypescriptableGraphQLType for TypeRef {
    fn recursive_typescriptify(
        &self,
        string: Option<String>,
        wrap_me: bool,
    ) -> Result<(String, bool)> {
        let (type_ref_string, wrap_me) = match self {
            TypeRef::Scalar { name } => (format!("{name}Scalar"), wrap_me),
            TypeRef::NonNull { of_type } => (*of_type).recursive_typescriptify(string, false)?,
            TypeRef::List { of_type } => {
                let (string, wrap_me) = (*of_type).recursive_typescriptify(string, wrap_me)?;
                (format!("{string}[]"), wrap_me)
            }
            type_ref => {
                let name = type_ref
                    .maybe_name()
                    .ok_or_else(|| eyre!("Tried to get name from nameless TypeRef"))?;
                (name.to_owned(), wrap_me)
            }
        };

        Ok(Self::wrap_if_nullable(type_ref_string, wrap_me))
    }
}

pub fn recursively_typescriptify_selected_object_fields<'a>(
    selection_set: SelectionSet<'a, &'a str>,
    buffer: &mut String,
    selectable_fields: &[Field],
    type_index: &TypeIndex,
) -> Result<()> {
    for selection in selection_set.items {
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
                    .find(|f| f.name == name)
                    .ok_or_else(|| eyre!("Tried to select non-existent field at {position}"))?;
                let field_name = alias.unwrap_or(&selected_field.name);

                write!(buffer, "{}: ", field_name)?;

                let mut nullable = true;
                recursively_typescriptify_selected_field(
                    selection_set,
                    buffer,
                    &selected_field.of_type,
                    type_index,
                    &mut nullable,
                )?;

                write!(buffer, ", ")?;
            }
            Selection::FragmentSpread(_) => todo!(),
            Selection::InlineFragment(_) => todo!(),
        }
    }

    Ok(())
}

fn recursively_typescriptify_selected_field<'a>(
    selection_set: SelectionSet<'a, &'a str>,
    buffer: &mut String,
    type_ref: &TypeRef,
    type_index: &TypeIndex,
    nullable: &mut bool,
) -> Result<()> {
    let selected_field_type = type_index.type_from_ref(type_ref);
    let mut local_buffer = String::new();

    match selected_field_type {
        Type::Object { fields, .. } => {
            write!(local_buffer, "{{ ")?;
            recursively_typescriptify_selected_object_fields(
                selection_set,
                &mut local_buffer,
                &fields,
                type_index,
            )?;
            write!(local_buffer, "}}")?;
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
            write!(local_buffer, "{leaf_field_type}")?;
        }
    };

    if *nullable {
        write!(buffer, "Nullable<{}>", local_buffer)?;
    } else {
        write!(buffer, "{}", local_buffer)?;
    }

    Ok(())
}
