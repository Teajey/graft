use std::path::{Path, PathBuf};

use ::graphql_parser::query::Type as GraphQLParserType;

use crate::graphql::schema::{NamedType, TypeRef};

pub type Arg<'a> = GraphQLParserType<'a, &'a str>;

pub trait MaybeNamed {
    fn maybe_name(&self) -> Option<&str>;
}

pub trait Named {
    fn name(&self) -> &str;
}

impl Named for NamedType {
    fn name(&self) -> &str {
        match self {
            NamedType::Scalar { name, .. }
            | NamedType::Object { name, .. }
            | NamedType::Interface { name, .. }
            | NamedType::Union { name, .. }
            | NamedType::Enum { name, .. }
            | NamedType::InputObject { name, .. } => name,
        }
    }
}

impl MaybeNamed for TypeRef {
    fn maybe_name(&self) -> Option<&str> {
        match self {
            TypeRef::To { name } => Some(name),
            _ => None,
        }
    }
}

// impl MaybeNamed for Type {
//     fn maybe_name(&self) -> Option<&str> {
//         match self {
//             Type::Scalar { name, .. }
//             | Type::Object { name, .. }
//             | Type::Interface { name, .. }
//             | Type::Union { name, .. }
//             | Type::Enum { name, .. }
//             | Type::InputObject { name, .. } => Some(name),
//             Type::NonNull { .. } | Type::List { .. } => None,
//         }
//     }
// }

pub fn path_with_possible_prefix(prefix: Option<&Path>, path: &Path) -> PathBuf {
    prefix
        .map(|p| p.join(path))
        .unwrap_or_else(|| PathBuf::from(path))
}

#[cfg(feature = "debug")]
pub mod debug {
    use eyre::{eyre, Result};
    use lazy_static::lazy_static;

    use crate::{cross, cross_eprintln};

    lazy_static! {
        static ref DO_DEBUG: bool = match init() {
            Ok(do_debug) => do_debug,
            Err(err) => panic!("Failed to init debug flag: {err}"),
        };
    }

    fn init() -> Result<bool> {
        let Some(accord_debug) = cross::env::option_var("ACCORD_DEBUG")? else {
            return Ok(false);
        };

        let result = match accord_debug.as_str() {
            "true" => Ok(true),
            "false" => Ok(false),
            value => Err(eyre!(
                "Invalid ACCORD_DEBUG value '{value}'. Must be 'true' or 'false'"
            )),
        };

        result
    }

    pub fn log(msg: &str) {
        {
            if *DO_DEBUG {
                cross_eprintln!("{} {msg}", console::style("DEBUG:").red());
            }
        }
    }
}

#[macro_export]
macro_rules! debug_log {
    ($($t:tt)*) => {
        #[cfg(feature = "debug")]
        $crate::util::debug::log(&format_args!($($t)*).to_string())
    }
}
