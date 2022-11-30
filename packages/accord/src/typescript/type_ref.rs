use eyre::{eyre, Result};

use super::Typescriptable;
use crate::introspection::TypeRef;
use crate::util::MaybeNamed;

impl Typescriptable for TypeRef {
    fn as_typescript(&self) -> Result<String> {
        recursive_typescriptify(self, &mut true)
    }

    fn as_typescript_non_nullable(&self) -> Result<String> {
        recursive_typescriptify(self, &mut false)
    }
}

fn recursive_typescriptify(type_ref: &TypeRef, nullable: &mut bool) -> Result<String> {
    let type_ref_string = match type_ref {
        TypeRef::Scalar { name } => format!("{name}Scalar"),
        TypeRef::NonNull { of_type } => {
            *nullable = false;
            recursive_typescriptify(of_type, nullable)?
        }
        TypeRef::List { of_type } => {
            let string = recursive_typescriptify(of_type, nullable)?;
            format!("{string}[]")
        }
        TypeRef::Union { name } => {
            format!("{name}Union")
        }
        TypeRef::Interface { name } => {
            format!("{name}Interface")
        }
        type_ref => {
            let name = type_ref
                .maybe_name()
                .ok_or_else(|| eyre!("Tried to get name from nameless TypeRef"))?;
            name.to_owned()
        }
    };

    let type_ref_string = if *nullable {
        format!("Nullable<{type_ref_string}>")
    } else {
        type_ref_string
    };

    Ok(type_ref_string)
}
