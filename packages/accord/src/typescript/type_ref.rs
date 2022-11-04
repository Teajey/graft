use eyre::{eyre, Result};

use super::Typescriptable;
use crate::introspection_response::TypeRef;
use crate::util::MaybeNamed;

impl Typescriptable for TypeRef {
    fn as_typescript(&self) -> Result<String> {
        recursive_typescriptify(self, &mut true)
    }
}

fn recursive_typescriptify(type_ref: &TypeRef, wrap_me: &mut bool) -> Result<String> {
    let type_ref_string = match type_ref {
        TypeRef::Scalar { name } => format!("{name}Scalar"),
        TypeRef::NonNull { of_type } => {
            *wrap_me = false;
            recursive_typescriptify(of_type, wrap_me)?
        }
        TypeRef::List { of_type } => {
            let string = recursive_typescriptify(of_type, wrap_me)?;
            format!("{string}[]")
        }
        type_ref => {
            let name = type_ref
                .maybe_name()
                .ok_or_else(|| eyre!("Tried to get name from nameless TypeRef"))?;
            name.to_owned()
        }
    };

    let string = if *wrap_me {
        format!("Nullable<{type_ref_string}>")
    } else {
        type_ref_string
    };

    Ok(string)
}
