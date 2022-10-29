use std::collections::HashMap;

use eyre::{eyre, Result};

use crate::introspection_response::TypeRef;
use crate::util::{Arg, MaybeNamed};

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

pub fn try_type_ref_from_arg<'a>(
    type_ref_index: &HashMap<&str, TypeRef>,
    arg: &Arg<'a>,
) -> Result<TypeRef> {
    match arg {
        Arg::NamedType(var_type_name) => type_ref_index
            .get(var_type_name)
            .ok_or_else(|| {
                eyre!(
                    "Found a query argument type not defined in the schema: {}",
                    var_type_name
                )
            })
            .map(|t| t.clone()),
        Arg::NonNullType(var_type) => {
            let type_ref = try_type_ref_from_arg(type_ref_index, var_type)?;
            Ok(TypeRef::NonNull {
                of_type: Box::new(type_ref),
            })
        }
        Arg::ListType(var_type) => {
            let type_ref = try_type_ref_from_arg(type_ref_index, var_type)?;
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
