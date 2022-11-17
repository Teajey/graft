use std::path::{Path, PathBuf};

use ::graphql_parser::query::Type as GraphQLParserType;

use crate::introspection::{Type, TypeRef};

pub type Arg<'a> = GraphQLParserType<'a, &'a str>;

pub trait MaybeNamed {
    fn maybe_name(&self) -> Option<&str>;
}

impl MaybeNamed for TypeRef {
    fn maybe_name(&self) -> Option<&str> {
        match self {
            TypeRef::Scalar { name }
            | TypeRef::Object { name }
            | TypeRef::Interface { name }
            | TypeRef::Union { name }
            | TypeRef::Enum { name }
            | TypeRef::InputObject { name } => Some(name),
            TypeRef::NonNull { .. } | TypeRef::List { .. } => None,
        }
    }
}

impl MaybeNamed for Type {
    fn maybe_name(&self) -> Option<&str> {
        match self {
            Type::Scalar { name, .. }
            | Type::Object { name, .. }
            | Type::Interface { name, .. }
            | Type::Union { name, .. }
            | Type::Enum { name, .. }
            | Type::InputObject { name, .. } => Some(name),
            Type::NonNull { .. } | Type::List { .. } => None,
        }
    }
}

pub fn path_with_possible_prefix(prefix: Option<&str>, path: &str) -> PathBuf {
    prefix
        .map(|p| Path::new(p).join(path))
        .unwrap_or_else(|| PathBuf::from(path))
}

pub mod env {
    use std::env::VarError;

    pub fn var(key: &str) -> Result<String, VarError> {
        #[cfg(feature = "native")]
        let value = std::env::var(key);
        #[cfg(feature = "node")]
        let value = crate::node::env::var(key);
        value
    }
}
