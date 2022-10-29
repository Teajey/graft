use ::graphql_parser::query::Type as GraphQLParserType;

use crate::introspection_response::{Type, TypeRef};

pub type Arg<'a> = GraphQLParserType<'a, &'a str>;

pub trait Named {
    fn name(&self) -> &str;
}

impl<'a> Named for Arg<'a> {
    fn name(&self) -> &str {
        match self {
            Arg::NamedType(name) => name,
            Arg::ListType(_) => "List",
            Arg::NonNullType(_) => "NonNull",
        }
    }
}

impl Named for Type {
    fn name(&self) -> &str {
        match self {
            Type::Scalar { name, .. } => name,
            Type::Object { name, .. } => name,
            Type::Interface { name, .. } => name,
            Type::Union { name, .. } => name,
            Type::Enum { name, .. } => name,
            Type::InputObject { name, .. } => name,
            Type::NonNull { .. } => "NonNull",
            Type::List { .. } => "List",
        }
    }
}

pub trait MaybeNamed {
    fn maybe_name(&self) -> Option<&str>;
}

impl MaybeNamed for TypeRef {
    fn maybe_name(&self) -> Option<&str> {
        match self {
            TypeRef::Scalar { name } => Some(name),
            TypeRef::Object { name } => Some(name),
            TypeRef::Interface { name } => Some(name),
            TypeRef::Union { name } => Some(name),
            TypeRef::Enum { name } => Some(name),
            TypeRef::InputObject { name } => Some(name),
            TypeRef::NonNull { .. } => None,
            TypeRef::List { .. } => None,
        }
    }
}
