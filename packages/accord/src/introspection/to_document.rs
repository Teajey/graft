use std::borrow::Borrow;

use graphql_parser::schema::{
    self as gql_parser, Definition, Document, InterfaceType, ObjectType, ScalarType,
    SchemaDefinition, TypeDefinition,
};

use super::{EnumValue, Field, InputValue, Schema, Type, TypeRef};
use crate::util::MaybeNamed;

impl<'a> From<&'a Schema> for Document<'a, &'a str> {
    fn from(schema: &'a Schema) -> Self {
        let schema_def = Definition::SchemaDefinition(SchemaDefinition {
            position: Default::default(),
            directives: vec![],
            query: Some(schema.query_type.name.as_str()),
            mutation: schema.mutation_type.as_ref().map(|m| m.name.as_str()),
            subscription: schema.subscription_type.as_ref().map(|s| s.name.as_str()),
        });

        let definitions = schema
            .types
            .iter()
            .filter(|t| !t.is_builtin())
            .map(|t| Definition::TypeDefinition(t.into()))
            .collect();

        let mut doc = Document { definitions };

        doc.definitions.push(schema_def);

        doc
    }
}

impl<'a> From<&'a EnumValue> for gql_parser::EnumValue<'a, &'a str> {
    fn from(value: &'a EnumValue) -> Self {
        Self {
            position: Default::default(),
            description: value.description.as_ref().cloned(),
            name: value.name.as_str(),
            directives: vec![],
        }
    }
}

impl<'a> From<&'a Type> for TypeDefinition<'a, &'a str> {
    fn from(t: &'a Type) -> Self {
        match t {
            Type::Scalar { name, description } => TypeDefinition::Scalar(ScalarType {
                position: Default::default(),
                description: description.as_ref().map(|d| d.as_str().into()),
                name: name.as_str(),
                directives: vec![],
            }),
            Type::Object {
                name,
                description,
                fields,
                interfaces,
            } => TypeDefinition::Object(ObjectType {
                position: Default::default(),
                description: description.as_ref().map(|d| d.as_str().into()),
                name: name.as_str(),
                implements_interfaces: interfaces.iter().filter_map(|o| o.maybe_name()).collect(),
                directives: vec![],
                fields: fields.iter().map(|f| f.into()).collect(),
            }),
            Type::Interface {
                name,
                description,
                fields,
                possible_types: _,
                interfaces,
            } => TypeDefinition::Interface(InterfaceType {
                position: Default::default(),
                description: description.as_ref().map(|d| d.as_str().into()),
                name: name.as_str(),
                implements_interfaces: interfaces.iter().filter_map(|o| o.maybe_name()).collect(),
                directives: vec![],
                fields: fields.iter().map(|f| f.into()).collect(),
            }),
            Type::Union {
                name,
                description,
                possible_types,
            } => TypeDefinition::Union(gql_parser::UnionType {
                position: Default::default(),
                description: description.as_ref().map(|d| d.as_str().into()),
                name: name.as_str(),
                directives: vec![],
                types: possible_types
                    .iter()
                    .filter_map(|o| o.maybe_name())
                    .collect(),
            }),
            Type::Enum {
                name,
                description,
                enum_values,
            } => TypeDefinition::Enum(gql_parser::EnumType {
                position: Default::default(),
                description: description.as_ref().cloned(),
                name: name.as_str(),
                directives: vec![],
                values: enum_values.iter().map(|e| e.into()).collect(),
            }),
            Type::InputObject {
                name,
                description,
                input_fields,
            } => TypeDefinition::InputObject(gql_parser::InputObjectType {
                position: Default::default(),
                description: description.as_ref().map(|d| d.as_str().into()),
                name: name.as_str(),
                directives: vec![],
                fields: input_fields.iter().map(|f| f.into()).collect(),
            }),
            Type::NonNull { .. } | Type::List { .. } => {
                panic!("graphql_parser::schema::TypeDefinition does not support NonNull and List")
            }
        }
    }
}

impl<'a> From<&'a InputValue> for gql_parser::InputValue<'a, &'a str> {
    fn from(input: &'a InputValue) -> Self {
        Self {
            position: Default::default(),
            description: input.description.as_ref().cloned(),
            name: input.name.as_str(),
            value_type: input.of_type.borrow().into(),
            default_value: input
                .default_value
                .as_ref()
                .map(|v| gql_parser::Value::String(v.clone())),
            directives: vec![],
        }
    }
}

impl<'a> From<&'a Field> for gql_parser::Field<'a, &'a str> {
    fn from(field: &'a Field) -> Self {
        Self {
            position: Default::default(),
            description: field.description.as_ref().cloned(),
            name: field.name.as_str(),
            arguments: field.args.iter().map(|a| a.into()).collect(),
            field_type: field.of_type.borrow().into(),
            directives: vec![],
        }
    }
}

impl<'a> From<&'a TypeRef> for gql_parser::Type<'a, &'a str> {
    fn from(type_ref: &'a TypeRef) -> Self {
        match type_ref {
            TypeRef::Scalar { name }
            | TypeRef::Object { name }
            | TypeRef::Interface { name }
            | TypeRef::Union { name }
            | TypeRef::Enum { name }
            | TypeRef::InputObject { name } => Self::NamedType(name.as_str()),
            TypeRef::NonNull { of_type } => Self::NonNullType(Box::new((&**of_type).into())),
            TypeRef::List { of_type } => Self::NonNullType(Box::new((&**of_type).into())),
        }
    }
}
