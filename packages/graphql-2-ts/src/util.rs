use std::collections::HashMap;

use ::graphql_parser::query::Type as GraphQLParserType;
use eyre::{eyre, Result};

use crate::introspection_response::{Schema, Type, TypeRef};

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
        match type_ref {
            TypeRef::NonNull { of_type } => Type::NonNull {
                of_type: (**of_type).clone(),
            },
            TypeRef::List { of_type } => Type::List {
                of_type: (**of_type).clone(),
            },
            TypeRef::Enum { name } 
            | TypeRef::InputObject { name } 
            | TypeRef::Interface { name } 
            | TypeRef::Object { name } 
            | TypeRef::Union { name } 
            | TypeRef::Scalar { name } => self.map.get(name).unwrap_or_else(|| {
                panic!(
                    "TypeIndex couldn't find the Type referred to by TypeRef::{:?}\nKeys available in TypeMap: {:#?}",
                    type_ref,
                    self.map.keys().collect::<Vec<_>>()
                )
            }).to_owned(),
        }
    }

    pub fn try_new(schema: &Schema) -> Result<Self> {
        let mut map = schema.types.iter().fold(HashMap::new(), |mut map, t| {
            if let Some(name) = t.maybe_name() {
                map.insert(name.to_owned(), (*t).clone());
            } else {
                eprintln!("WARN: TypeIndex tried to index an unnamed type.");
            }
            map
        });
        let query = map
            .remove(&schema.query_type.name)
            .ok_or_else(|| eyre!("TypeIndex has no query type"))?;
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
