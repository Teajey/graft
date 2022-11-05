use eyre::{eyre, Result};
use graphql_client::GraphQLQuery;
use serde::{Deserialize, Serialize};

use crate::config::AppConfig;
use crate::typescript::TypeIndex;
use crate::util::Arg;

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
    pub fn try_from_arg<'a>(type_index: &TypeIndex, arg: &Arg<'a>) -> Result<TypeRef> {
        match arg {
            Arg::NamedType(var_type_name) => type_index
                .get(var_type_name)
                .ok_or_else(|| {
                    eyre!(
                        "Found a query argument type not defined in the schema: {}",
                        var_type_name
                    )
                })
                .map(|t| t.clone().into()),
            Arg::NonNullType(var_type) => {
                let type_ref = Self::try_from_arg(type_index, var_type)?;
                Ok(TypeRef::NonNull {
                    of_type: Box::new(type_ref),
                })
            }
            Arg::ListType(var_type) => {
                let type_ref = Self::try_from_arg(type_index, var_type)?;
                Ok(TypeRef::List {
                    of_type: Box::new(type_ref),
                })
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
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
        interfaces: Vec<TypeRef>,
    },
    #[serde(rename_all = "camelCase")]
    Interface {
        name: String,
        description: Option<String>,
        fields: Vec<Field>,
        possible_types: Vec<TypeRef>,
    },
    #[serde(rename_all = "camelCase")]
    Union {
        name: String,
        description: Option<String>,
        possible_types: Vec<TypeRef>,
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
        input_fields: Vec<InputValue>,
    },
    #[serde(rename_all = "camelCase")]
    NonNull { of_type: TypeRef },
    #[serde(rename_all = "camelCase")]
    List { of_type: TypeRef },
}

impl From<Type> for TypeRef {
    fn from(other: Type) -> Self {
        match other {
            Type::Scalar { name, .. } => TypeRef::Scalar { name },
            Type::Object { name, .. } => TypeRef::Object { name },
            Type::Interface { name, .. } => TypeRef::Interface { name },
            Type::Union { name, .. } => TypeRef::Union { name },
            Type::Enum { name, .. } => TypeRef::Enum { name },
            Type::InputObject { name, .. } => TypeRef::InputObject { name },
            Type::NonNull { of_type } => TypeRef::NonNull {
                of_type: Box::new(of_type),
            },
            Type::List { of_type } => TypeRef::List {
                of_type: Box::new(of_type),
            },
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
    pub default_value: Option<String>,
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

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/graphql/introspection_schema.graphql",
    query_path = "src/graphql/introspection_query.graphql",
    response_derives = "Serialize",
    variable_derives = "Deserialize"
)]
struct IntrospectionQuery;

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    pub data: Data,
}

impl Response {
    pub async fn fetch(config: &AppConfig) -> Result<Self> {
        let body = IntrospectionQuery::build_query(introspection_query::Variables {});

        let client = reqwest::Client::builder()
            .danger_accept_invalid_certs(config.no_ssl.unwrap_or(false))
            .build()?;

        let res = client
            .post(config.schema.clone())
            .json(&body)
            .send()
            .await?;

        Ok(res.json().await?)
    }
}
