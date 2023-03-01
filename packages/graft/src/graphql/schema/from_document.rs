use eyre::{eyre, Report, Result};
use graphql_parser::schema as gp;

use crate::graphql::schema as ac;

impl From<gp::Type<'_, String>> for ac::TypeRef {
    fn from(value: gp::Type<'_, String>) -> Self {
        match value {
            gp::Type::NamedType(name) => Self::To { name },
            gp::Type::ListType(of_type) => Self::Container(ac::TypeRefContainer::List {
                of_type: Box::new((*of_type).into()),
            }),
            gp::Type::NonNullType(of_type) => Self::Container(ac::TypeRefContainer::NonNull {
                of_type: Box::new((*of_type).into()),
            }),
        }
    }
}

impl From<gp::InputValue<'_, String>> for ac::InputValue {
    fn from(
        gp::InputValue {
            position: _,
            description,
            name,
            value_type,
            default_value: _,
            // Reading directives with introspection not supported: https://stackoverflow.com/a/65064958/2269124
            directives: _,
        }: gp::InputValue<'_, String>,
    ) -> Self {
        Self {
            name,
            description,
            of_type: value_type.into(),
        }
    }
}

impl From<gp::Field<'_, String>> for ac::Field {
    fn from(
        gp::Field {
            position: _,
            description,
            name,
            arguments,
            field_type,
            directives: _,
        }: gp::Field<'_, String>,
    ) -> Self {
        Self {
            name,
            description,
            args: arguments.into_iter().map(Into::into).collect(),
            of_type: field_type.into(),
            is_deprecated: false,
            deprecation_reason: None,
        }
    }
}

impl From<gp::EnumValue<'_, String>> for ac::EnumValue {
    fn from(
        gp::EnumValue {
            position: _,
            description,
            name,
            directives: _,
        }: gp::EnumValue<'_, String>,
    ) -> Self {
        Self {
            name,
            description,
            is_deprecated: false,
            deprecation_reason: None,
        }
    }
}

impl From<gp::TypeDefinition<'_, String>> for ac::NamedType {
    fn from(value: gp::TypeDefinition<'_, String>) -> Self {
        match value {
            gp::TypeDefinition::Scalar(gp::ScalarType {
                position: _,
                description,
                name,
                directives: _,
            }) => ac::NamedType::Scalar { name, description },
            gp::TypeDefinition::Object(gp::ObjectType {
                position: _,
                description,
                name,
                implements_interfaces,
                directives: _,
                fields,
            }) => ac::NamedType::Object {
                name,
                description,
                fields: fields.into_iter().map(Into::into).collect(),
                interfaces: implements_interfaces
                    .into_iter()
                    .map(|name| ac::TypeRef::To { name })
                    .collect(),
            },
            gp::TypeDefinition::Interface(gp::InterfaceType {
                position: _,
                description,
                name,
                implements_interfaces,
                directives: _,
                fields,
            }) => ac::NamedType::Interface {
                name,
                description,
                fields: fields.into_iter().map(Into::into).collect(),
                possible_types: vec![],
                interfaces: implements_interfaces
                    .into_iter()
                    .map(|name| ac::TypeRef::To { name })
                    .collect(),
            },
            gp::TypeDefinition::Union(gp::UnionType {
                position: _,
                description,
                name,
                directives: _,
                types,
            }) => ac::NamedType::Union {
                name,
                description,
                possible_types: types
                    .into_iter()
                    .map(|name| ac::TypeRef::To { name })
                    .collect(),
            },
            gp::TypeDefinition::Enum(gp::EnumType {
                position: _,
                description,
                name,
                directives: _,
                values,
            }) => ac::NamedType::Enum {
                name,
                description,
                enum_values: values.into_iter().map(Into::into).collect(),
            },
            gp::TypeDefinition::InputObject(gp::InputObjectType {
                position: _,
                description,
                name,
                directives: _,
                fields,
            }) => ac::NamedType::InputObject {
                name,
                description,
                input_fields: fields.into_iter().map(Into::into).collect(),
            },
        }
    }
}

impl From<gp::DirectiveLocation> for ac::DirectiveLocation {
    fn from(value: gp::DirectiveLocation) -> Self {
        use ac::DirectiveLocation as ac;
        use gp::DirectiveLocation as gp;
        match value {
            gp::Query => ac::Query,
            gp::Mutation => ac::Mutation,
            gp::Subscription => ac::Subscription,
            gp::Field => ac::Field,
            gp::FragmentDefinition => ac::FragmentDefinition,
            gp::FragmentSpread => ac::FragmentSpread,
            gp::InlineFragment => ac::InlineFragment,
            gp::Schema => ac::Schema,
            gp::Scalar => ac::Scalar,
            gp::Object => ac::Object,
            gp::FieldDefinition => ac::FieldDefinition,
            gp::ArgumentDefinition => ac::ArgumentDefinition,
            gp::Interface => ac::Interface,
            gp::Union => ac::Union,
            gp::Enum => ac::Enum,
            gp::EnumValue => ac::EnumValue,
            gp::InputObject => ac::InputObject,
            gp::InputFieldDefinition => ac::InputFieldDefinition,
        }
    }
}

impl From<gp::DirectiveDefinition<'_, String>> for ac::Directive {
    fn from(
        gp::DirectiveDefinition {
            position: _,
            description,
            name,
            arguments,
            repeatable,
            locations,
        }: gp::DirectiveDefinition<'_, String>,
    ) -> Self {
        Self {
            description,
            name,
            locations: locations.into_iter().map(Into::into).collect(),
            args: arguments.into_iter().map(Into::into).collect(),
        }
    }
}

impl TryFrom<gp::Document<'_, String>> for ac::Schema {
    type Error = Report;

    fn try_from(document: gp::Document<'_, String>) -> Result<Self> {
        let mut query_type = Option::<ac::RootType>::None;
        let mut mutation_type = Option::<ac::RootType>::None;
        let mut subscription_type = Option::<ac::RootType>::None;

        let mut types = Vec::<ac::NamedType>::new();
        let mut directives = Vec::<ac::Directive>::new();

        for def in document.definitions {
            match def {
                gp::Definition::SchemaDefinition(gp::SchemaDefinition {
                    position: _,
                    directives: _,
                    query,
                    mutation,
                    subscription,
                }) => {
                    if query_type.is_some() {
                        return Err(eyre!("Tried to set schema definition more than once"));
                    }
                    query_type = query.map(|name| ac::RootType { name });
                    mutation_type = mutation.map(|name| ac::RootType { name });
                    subscription_type = subscription.map(|name| ac::RootType { name });
                }
                gp::Definition::TypeDefinition(type_definition) => {
                    types.push(type_definition.into());
                }
                gp::Definition::TypeExtension(_) => panic!("TypeExtension not yet supported"),
                gp::Definition::DirectiveDefinition(directive_definition) => {
                    directives.push(directive_definition.into());
                }
            }
        }

        let query_type = query_type.ok_or_else(|| eyre!("query_type was not filled"))?;

        Ok(ac::Schema {
            types,
            query_type,
            mutation_type,
            subscription_type,
            directives,
        })
    }
}
