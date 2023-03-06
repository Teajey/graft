use eyre::{eyre, ErrReport, Report, Result};
use graphql_parser::schema as gp;

use crate::graphql::schema as ac;

impl TryFrom<gp::Type<'_, String>> for ac::NonNullTypeRef {
    type Error = ErrReport;

    fn try_from(value: gp::Type<'_, String>) -> std::result::Result<Self, Self::Error> {
        let type_ref = match value {
            gp::Type::NamedType(name) => Self::To { name },
            gp::Type::ListType(list) => Self::Container(ac::NonNullTypeRefContainer::List {
                of_type: Box::new((*list).try_into()?),
            }),
            gp::Type::NonNullType(_) => return Err(eyre!("NonNull cannot contain NonNull")),
        };

        Ok(type_ref)
    }
}

impl TryFrom<gp::Type<'_, String>> for ac::TypeRef {
    type Error = ErrReport;

    fn try_from(value: gp::Type<'_, String>) -> Result<Self> {
        let type_ref = match value {
            gp::Type::NamedType(name) => Self::To { name },
            gp::Type::ListType(of_type) => Self::Container(ac::TypeRefContainer::List {
                of_type: Box::new((*of_type).try_into()?),
            }),
            gp::Type::NonNullType(of_type) => Self::Container(ac::TypeRefContainer::NonNull {
                of_type: Box::new((*of_type).try_into()?),
            }),
        };

        Ok(type_ref)
    }
}

impl TryFrom<gp::InputValue<'_, String>> for ac::InputValue {
    type Error = ErrReport;

    fn try_from(
        gp::InputValue {
            position: _,
            description,
            name,
            value_type,
            default_value: _,
            // Reading directives with introspection not supported: https://stackoverflow.com/a/65064958/2269124
            directives: _,
        }: gp::InputValue<'_, String>,
    ) -> Result<Self> {
        Ok(Self {
            name,
            description,
            of_type: value_type.try_into()?,
        })
    }
}

impl TryFrom<gp::Field<'_, String>> for ac::Field {
    type Error = ErrReport;

    fn try_from(
        gp::Field {
            position: _,
            description,
            name,
            arguments,
            field_type,
            directives: _,
        }: gp::Field<'_, String>,
    ) -> Result<Self> {
        Ok(Self {
            name,
            description,
            args: arguments
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_>>()?,
            of_type: field_type.try_into()?,
            is_deprecated: false,
            deprecation_reason: None,
        })
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

impl TryFrom<gp::TypeDefinition<'_, String>> for ac::NamedType {
    type Error = ErrReport;

    fn try_from(value: gp::TypeDefinition<'_, String>) -> Result<Self> {
        use ac::named_type::{Enum, InputObject, Interface, Object, Scalar, Union};

        let named_type = match value {
            gp::TypeDefinition::Scalar(gp::ScalarType {
                position: _,
                description,
                name,
                directives: _,
            }) => ac::NamedType::Scalar(Scalar { name, description }),
            gp::TypeDefinition::Object(gp::ObjectType {
                position: _,
                description,
                name,
                implements_interfaces,
                directives: _,
                fields,
            }) => ac::NamedType::Object(Object {
                name,
                description,
                fields: fields
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<_>>()?,
                interfaces: implements_interfaces
                    .into_iter()
                    .map(|name| ac::TypeRef::To { name })
                    .collect(),
            }),
            gp::TypeDefinition::Interface(gp::InterfaceType {
                position: _,
                description,
                name,
                implements_interfaces,
                directives: _,
                fields,
            }) => ac::NamedType::Interface(Interface {
                name,
                description,
                fields: fields
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<_>>()?,
                possible_types: vec![],
                interfaces: implements_interfaces
                    .into_iter()
                    .map(|name| ac::TypeRef::To { name })
                    .collect(),
            }),
            gp::TypeDefinition::Union(gp::UnionType {
                position: _,
                description,
                name,
                directives: _,
                types,
            }) => ac::NamedType::Union(Union {
                name,
                description,
                possible_types: types
                    .into_iter()
                    .map(|name| ac::TypeRef::To { name })
                    .collect(),
            }),
            gp::TypeDefinition::Enum(gp::EnumType {
                position: _,
                description,
                name,
                directives: _,
                values,
            }) => ac::NamedType::Enum(Enum {
                name,
                description,
                enum_values: values.into_iter().map(Into::into).collect(),
            }),
            gp::TypeDefinition::InputObject(gp::InputObjectType {
                position: _,
                description,
                name,
                directives: _,
                fields,
            }) => ac::NamedType::InputObject(InputObject {
                name,
                description,
                input_fields: fields
                    .into_iter()
                    .map(TryInto::try_into)
                    .collect::<Result<_>>()?,
            }),
        };

        Ok(named_type)
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

impl TryFrom<gp::DirectiveDefinition<'_, String>> for ac::Directive {
    type Error = ErrReport;

    fn try_from(
        gp::DirectiveDefinition {
            position: _,
            description,
            name,
            arguments,
            // Not supported in GraphQL 2018
            repeatable: _,
            locations,
        }: gp::DirectiveDefinition<'_, String>,
    ) -> Result<Self> {
        Ok(Self {
            description,
            name,
            locations: locations.into_iter().map(Into::into).collect(),
            args: arguments
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<_>>()?,
        })
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
                    types.push(type_definition.try_into()?);
                }
                gp::Definition::TypeExtension(_) => panic!("TypeExtension not yet supported"),
                gp::Definition::DirectiveDefinition(directive_definition) => {
                    directives.push(directive_definition.try_into()?);
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
