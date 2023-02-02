pub mod definition;
pub mod graphql_type;
pub mod type_ref;

use std::collections::HashMap;
use std::fmt::Write;

use eyre::{eyre, Result};

use crate::gen::Buffer;
use crate::introspection::{NamedType, Schema, Type, TypeRef};
use crate::util::Named;

pub(in crate::typescript) fn possibly_write_description<W: Write>(
    out: &mut W,
    description: Option<&String>,
) -> Result<()> {
    if let Some(description) = description {
        if description.contains('\n') {
            writeln!(out, "/**\n * {}\n */", description.replace('\n', "\n * "))?;
        } else {
            writeln!(out, "/** {} */", description)?;
        }
    };

    Ok(())
}

pub struct TypeIndex<'a> {
    map: HashMap<String, &'a NamedType>,
    pub query: &'a NamedType,
    pub mutation: Option<&'a NamedType>,
    pub subscription: Option<&'a NamedType>,
}

impl<'a> TypeIndex<'a> {
    pub fn get(&self, k: &str) -> Option<&NamedType> {
        self.map.get(k).copied()
    }

    pub fn type_from_ref(&self, type_ref: TypeRef) -> Result<Type> {
        let t = match type_ref {
            TypeRef::Container(contained) => Type::Container(contained),
            TypeRef::To { name } => {
                let named_type = self.map.get(&name)
                .copied().ok_or_else(|| eyre!(
                    "TypeIndex couldn't find the Type referred to by TypeRef::{{ name: {:?} }}\nKeys available in TypeMap: {:#?}",
                    name,
                    self.map.keys().collect::<Vec<_>>()))?;
                Type::Named(named_type.clone())
            }
        };

        Ok(t)
    }

    pub fn try_new(schema: &'a Schema) -> Result<Self> {
        let mut map = schema.types.iter().fold(HashMap::new(), |mut map, t| {
            map.insert(t.name().to_owned(), t);
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

    pub fn with<'b, 'c, T>(&'b self, target: &'c T) -> WithIndex<'c, 'b, 'a, T> {
        WithIndex {
            target,
            type_index: self,
        }
    }
}

pub struct WithIndex<'a, 'b, 'c, T> {
    target: &'a T,
    type_index: &'b TypeIndex<'c>,
}

pub trait WithIndexable: Sized {
    fn with_index<'a, 'b, 'c>(
        &'a self,
        type_index: &'b TypeIndex<'c>,
    ) -> WithIndex<'a, 'b, 'c, Self> {
        WithIndex {
            target: self,
            type_index,
        }
    }
}

pub trait Typescriptable {
    fn as_typescript(&self) -> Result<String>;

    fn as_typescript_non_nullable(&self) -> Result<String> {
        unimplemented!()
    }
}

pub(crate) trait TypescriptableWithBuffer {
    fn as_typescript_on(&self, buffer: &mut Buffer) -> Result<()>;
}
