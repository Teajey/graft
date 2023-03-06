mod from_document;
mod to_document;

use eyre::{eyre, Result};
use serde::{Deserialize, Serialize};

use crate::{graphql::query, util::Named};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EnumValue {
    pub name: String,
    pub description: Option<String>,
    pub is_deprecated: bool,
    pub deprecation_reason: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TypeKind {
    Scalar,
    Object,
    Interface,
    Union,
    Enum,
    InputObject,
    List,
    NonNull,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NonNullTypeRefContainer {
    #[serde(rename_all = "camelCase")]
    List { of_type: Box<TypeRef> },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum NonNullTypeRef {
    Container(NonNullTypeRefContainer),
    To { name: String },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TypeRefContainer {
    #[serde(rename_all = "camelCase")]
    NonNull { of_type: Box<NonNullTypeRef> },
    #[serde(rename_all = "camelCase")]
    List { of_type: Box<TypeRef> },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum TypeRef {
    Container(TypeRefContainer),
    To { name: String },
}

impl TypeRef {
    pub fn is_non_null(&self) -> bool {
        matches!(self, TypeRef::Container(TypeRefContainer::NonNull { .. }))
    }
}

impl From<Type> for TypeRef {
    fn from(value: Type) -> Self {
        match value {
            Type::Container(tfc) => TypeRef::Container(tfc),
            Type::Named(named) => TypeRef::To {
                name: named.name().to_owned(),
            },
        }
    }
}

impl From<&query::NonNullType> for NonNullTypeRef {
    fn from(value: &query::NonNullType) -> Self {
        match value {
            query::NonNullType::Named { name } => NonNullTypeRef::To {
                name: name.to_string(),
            },
            query::NonNullType::List { value } => {
                let type_ref = (&**value).into();
                NonNullTypeRef::Container(NonNullTypeRefContainer::List {
                    of_type: Box::new(type_ref),
                })
            }
        }
    }
}

impl From<&query::Type> for TypeRef {
    fn from(value: &query::Type) -> Self {
        match value {
            query::Type::Named { name } => TypeRef::To {
                name: name.to_string(),
            },
            query::Type::NonNull { value } => {
                let type_ref = value.into();
                TypeRef::Container(TypeRefContainer::NonNull {
                    of_type: Box::new(type_ref),
                })
            }
            query::Type::List { value } => {
                let type_ref = (&**value).into();
                TypeRef::Container(TypeRefContainer::List {
                    of_type: Box::new(type_ref),
                })
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Type {
    Container(TypeRefContainer),
    Named(NamedType),
}

impl Type {
    pub fn try_into_named(self) -> Result<NamedType> {
        match self {
            Type::Container(_) => Err(eyre!("Tried to get {self:?} as NamedType")),
            Type::Named(named_type) => Ok(named_type),
        }
    }

    pub fn into_type_ref(self) -> TypeRef {
        self.into()
    }
}

pub mod named_type {
    use serde::{Deserialize, Serialize};

    use super::{EnumValue, Field, InputValue, TypeRef};

    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct Scalar {
        pub name: String,
        pub description: Option<String>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct Object {
        pub name: String,
        pub description: Option<String>,
        pub fields: Vec<Field>,
        pub interfaces: Vec<TypeRef>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct Interface {
        pub name: String,
        pub description: Option<String>,
        pub fields: Vec<Field>,
        pub possible_types: Vec<TypeRef>,
        // FIXME: this field only valid in the October 2021 GraphQL spec
        #[serde(skip)]
        pub interfaces: Vec<TypeRef>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct Union {
        pub name: String,
        pub description: Option<String>,
        pub possible_types: Vec<TypeRef>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct Enum {
        pub name: String,
        pub description: Option<String>,
        pub enum_values: Vec<EnumValue>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(rename_all = "camelCase")]
    pub struct InputObject {
        pub name: String,
        pub description: Option<String>,
        pub input_fields: Vec<InputValue>,
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NamedType {
    Scalar(named_type::Scalar),
    Object(named_type::Object),
    Interface(named_type::Interface),
    Union(named_type::Union),
    Enum(named_type::Enum),
    InputObject(named_type::InputObject),
}

impl NamedType {
    pub fn is_internal(&self) -> bool {
        self.name().starts_with("__")
    }

    pub fn to_type_ref(&self) -> TypeRef {
        self.into()
    }
}

impl From<&NamedType> for TypeRef {
    fn from(t: &NamedType) -> Self {
        TypeRef::To {
            name: t.name().to_owned(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct InputValue {
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub of_type: TypeRef,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Field {
    pub name: String,
    pub description: Option<String>,
    pub args: Vec<InputValue>,
    #[serde(rename = "type")]
    pub of_type: TypeRef,
    pub is_deprecated: bool,
    pub deprecation_reason: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DirectiveLocation {
    Query,
    Mutation,
    Subscription,
    Field,
    FragmentDefinition,
    FragmentSpread,
    InlineFragment,
    Schema,
    Scalar,
    Object,
    FieldDefinition,
    ArgumentDefinition,
    Interface,
    Union,
    Enum,
    EnumValue,
    InputObject,
    InputFieldDefinition,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Directive {
    pub description: Option<String>,
    pub name: String,
    pub locations: Vec<DirectiveLocation>,
    pub args: Vec<InputValue>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RootType {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Schema {
    pub types: Vec<NamedType>,
    pub query_type: RootType,
    pub mutation_type: Option<RootType>,
    pub subscription_type: Option<RootType>,
    pub directives: Vec<Directive>,
}
