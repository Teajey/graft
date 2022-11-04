pub mod definition;
pub mod graphql_type;
pub mod type_ref;

use std::collections::HashMap;
use std::fmt::Write;

use eyre::{eyre, Result};

use crate::introspection::{Schema, Type, TypeRef};
use crate::util::MaybeNamed;
use crate::Buffer;

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

pub struct WithIndex<'a, 'b, T> {
    target: &'a T,
    type_index: &'b TypeIndex,
}

pub trait WithIndexable: Sized {
    fn with_index<'a, 'b>(&'a self, type_index: &'b TypeIndex) -> WithIndex<'a, 'b, Self> {
        WithIndex { target: self, type_index }
    }
}

pub trait Typescriptable {
    fn as_typescript(&self) -> Result<String>;
}

pub trait TypescriptableWithBuffer<'a> {
    fn as_typescript_on(&'a self, buffer: &mut Buffer) -> Result<()>;
}
