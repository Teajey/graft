use eyre::Result;

use super::{Typescriptable, WithIndex};
use crate::introspection::{Type, TypeRef, TypeRefContainer};

impl<'a, 'b, 'c> Typescriptable for WithIndex<'a, 'b, 'c, TypeRef> {
    fn as_typescript(&self) -> Result<String> {
        recursive_typescriptify(self, &mut true)
    }

    fn as_typescript_non_nullable(&self) -> Result<String> {
        recursive_typescriptify(self, &mut false)
    }
}

fn recursive_typescriptify<'a, 'b, 'c>(
    type_ref_with_index: &WithIndex<'a, 'b, 'c, TypeRef>,
    nullable: &mut bool,
) -> Result<String> {
    let WithIndex { target, type_index } = type_ref_with_index;
    let this_type = type_index.type_from_ref((*target).clone())?;
    let type_name = match this_type {
        Type::Container(TypeRefContainer::NonNull { of_type }) => {
            *nullable = false;
            recursive_typescriptify(&type_index.with(&of_type), nullable)?
        }
        Type::Container(TypeRefContainer::List { of_type }) => {
            let string = recursive_typescriptify(&type_index.with(&of_type), nullable)?;
            format!("{string}[]")
        }
        Type::Named(other_type) => other_type.typescript_name(),
    };

    let type_name = if *nullable {
        format!("Nullable<{type_name}>")
    } else {
        type_name
    };

    Ok(type_name)
}
