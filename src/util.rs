use std::collections::HashMap;

use ::graphql_parser::query::Type as GraphQLParserType;
use eyre::{eyre, Result};

use crate::introspection_response::{Schema, Type, TypeRef};

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

impl Named for TypeRef {
    fn name(&self) -> &str {
        match self {
            TypeRef::Scalar { name, .. } => name,
            TypeRef::Object { name, .. } => name,
            TypeRef::Interface { name, .. } => name,
            TypeRef::Union { name, .. } => name,
            TypeRef::Enum { name, .. } => name,
            TypeRef::InputObject { name, .. } => name,
            TypeRef::NonNull { .. } => "NonNull",
            TypeRef::List { .. } => "List",
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

pub trait OkOrElse {
    fn ok_or_else<E, F>(self, err: F) -> std::result::Result<(), E>
    where
        F: FnOnce() -> E,
        Self: Sized;
}

impl OkOrElse for bool {
    fn ok_or_else<E, F>(self, err: F) -> std::result::Result<(), E>
    where
        F: FnOnce() -> E,
        Self: Sized,
    {
        if !self {
            Err(err())
        } else {
            Ok(())
        }
    }
}

// TODO: `Type` should probably be `&Type`
pub struct TypeIndex {
    map: HashMap<String, Type>,
    pub query: Type,
    pub mutation: Option<Type>,
    pub subscription: Option<Type>,
}

impl TypeIndex {
    pub fn get(&self, k: &str) -> Option<&Type> {
        self.map.get(k)
    }

    pub fn type_from_ref(&self, type_ref: &TypeRef) -> Type {
        let resolve_or_panic = |type_ref: &TypeRef| {
            self.map.get(type_ref.name()).unwrap_or_else(|| {
                panic!(
                    "TypeIndex couldn't find the Type referred to by TypeRef::{:?}\nKeys available in TypeMap: {:#?}",
                    type_ref,
                    self.map.keys().collect::<Vec<_>>()
                )
            }).to_owned()
        };

        match type_ref {
            TypeRef::NonNull { of_type } => {
                resolve_or_panic(of_type);
                Type::NonNull {
                    of_type: (**of_type).clone(),
                }
            }
            TypeRef::List { of_type } => {
                resolve_or_panic(of_type);
                Type::List {
                    of_type: (**of_type).clone(),
                }
            }
            type_ref => resolve_or_panic(type_ref),
        }
    }

    pub fn try_new(schema: &Schema) -> Result<Self> {
        let mut map = schema.types.iter().fold(HashMap::new(), |mut map, t| {
            map.insert(t.name().to_owned(), (*t).clone());
            map
        });
        let query = map
            .remove(&schema.query_type.name)
            .ok_or_else(|| eyre!("Type Index has no query type"))?;
        let mutation = schema
            .mutation_type
            .as_ref()
            .and_then(|mutation_type| map.remove(&mutation_type.name));
        let subscription = schema
            .subscription_type
            .as_ref()
            .and_then(|subscription_type| map.remove(&subscription_type.name));
        Ok(Self {
            map,
            query,
            mutation,
            subscription,
        })
    }
}
