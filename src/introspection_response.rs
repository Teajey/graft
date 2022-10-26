use std::fmt::Display;

use eyre::{eyre, Result};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
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
pub enum TypeRef {
    Scalar {
        name: String,
    },
    Object {
        name: String,
    },
    Interface {
        name: String,
    },
    Union {
        name: String,
    },
    Enum {
        name: String,
    },
    InputObject {
        name: String,
    },
    #[serde(rename_all = "camelCase")]
    NonNull {
        of_type: Box<TypeRef>,
    },
    #[serde(rename_all = "camelCase")]
    List {
        of_type: Box<TypeRef>,
    },
}

impl TypeRef {
    pub fn name(&self) -> Option<&str> {
        match self {
            TypeRef::Scalar { name } => Some(name),
            TypeRef::Object { name } => Some(name),
            TypeRef::Interface { name } => Some(name),
            TypeRef::Union { name } => Some(name),
            TypeRef::Enum { name } => Some(name),
            TypeRef::InputObject { name } => Some(name),
            TypeRef::NonNull { of_type: _ } => None,
            TypeRef::List { of_type: _ } => None,
        }
    }

    fn stringify_as_typescript_type(
        &self,
        string: Option<String>,
        wrap_me: bool,
    ) -> Result<(String, bool)> {
        let (type_ref_string, wrap_me) = match self {
            TypeRef::Scalar { name } => (format!("{name}Scalar"), wrap_me),
            TypeRef::NonNull { of_type } => {
                (*of_type).stringify_as_typescript_type(string, false)?
            }
            TypeRef::List { of_type } => {
                let (string, wrap_me) = (*of_type).stringify_as_typescript_type(string, wrap_me)?;
                (format!("{string}[]"), wrap_me)
            }
            type_ref => {
                let name = type_ref
                    .name()
                    .ok_or_else(|| eyre!("Tried to get name from nameless TypeRef"))?;
                (name.to_owned(), wrap_me)
            }
        };

        if wrap_me {
            Ok((format!("Nullable<{type_ref_string}>"), wrap_me))
        } else {
            Ok((type_ref_string, wrap_me))
        }
    }
}

impl Display for TypeRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.stringify_as_typescript_type(None, true) {
            Ok((string, _)) => write!(f, "{}", string),
            Err(_) => Err(std::fmt::Error),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Type {
    #[serde(rename_all = "camelCase")]
    Scalar {
        name: String,
        description: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    Object {
        name: String,
        description: Option<String>,
        fields: Vec<Field>,
        interfaces: Vec<Type>,
    },
    #[serde(rename_all = "camelCase")]
    Interface {
        name: String,
        description: Option<String>,
        fields: Vec<Field>,
        possible_types: Option<Box<Type>>,
    },
    #[serde(rename_all = "camelCase")]
    Union {
        name: String,
        description: Option<String>,
        possible_types: Option<Box<Type>>,
    },
    #[serde(rename_all = "camelCase")]
    Enum {
        name: String,
        description: Option<String>,
        enum_values: Vec<EnumValue>,
    },
    #[serde(rename_all = "camelCase")]
    InputObject {
        name: String,
        description: Option<String>,
        input_fields: Option<Vec<InputValue>>,
    },
    #[serde(rename_all = "camelCase")]
    NonNull { of_type: Option<TypeRef> },
    #[serde(rename_all = "camelCase")]
    List { of_type: Option<TypeRef> },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InputValue {
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub of_type: TypeRef,
    pub default_value: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
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
    pub types: Vec<Type>,
    pub query_type: RootType,
    pub mutation_type: Option<RootType>,
    pub subscription_type: Option<RootType>,
    pub directives: Vec<Directive>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Data {
    #[serde(rename = "__schema")]
    pub schema: Schema,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IntrospectionResponse {
    pub data: Data,
}
