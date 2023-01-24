use std::path::{Path, PathBuf};

use ::graphql_parser::query::Type as GraphQLParserType;
use eyre::Result;

use crate::{
    cross,
    introspection::{Type, TypeRef},
};

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

pub fn path_with_possible_prefix(prefix: Option<&Path>, path: &Path) -> PathBuf {
    prefix
        .map(|p| p.join(path))
        .unwrap_or_else(|| PathBuf::from(path))
}

pub fn file_location_in_path_by_prefix(prefix: &str) -> Result<Option<PathBuf>> {
    fn recurse(prefix: &str, current_dir: &Path) -> Result<Option<PathBuf>> {
        let file_names = cross::fs::read_dir(current_dir)?;

        let prefix_matched = file_names.into_iter().any(|f| f.starts_with(prefix));

        if prefix_matched {
            Ok(Some(current_dir.to_path_buf()))
        } else if let Some(dir) = current_dir.parent() {
            recurse(prefix, dir)
        } else {
            Ok(None)
        }
    }

    recurse(prefix, &cross::env::current_dir()?)
}

pub mod debug {
    use eyre::{eyre, Result};

    use crate::{cross, cross_eprint};

    pub fn log(msg: &str) -> Result<()> {
        let do_debug = cross::env::var("ACCORD_DEBUG")
            .ok()
            .map(|loo| match loo.as_str() {
                "true" => Some(true),
                "false" => Some(false),
                _ => None,
            });
        match do_debug {
            Some(Some(do_debug)) => {
                if do_debug {
                    cross_eprint!("{msg}");
                }
                Ok(())
            }
            None => Ok(()),
            _ => Err(eyre!(
                "Invalid ACCORD_DEBUG value. Must be 'true' or 'false'"
            )),
        }
    }
}
