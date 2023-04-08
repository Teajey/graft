use std::collections::HashMap;

use eyre::{eyre, Report, Result};

use crate::{app::config::TypescriptOptions, graphql::schema as gql, typescript::ts2 as ts};

impl From<gql::TypeRef> for ts::InterfaceRef {
    fn from(value: gql::TypeRef) -> Self {
        match value {
            gql::TypeRef::Container(_) => {
                // Maybe this is still possible at runtime? In which case it should be
                // an actual error.
                unreachable!("Object.interfaces element can't be List or NonNull")
            }
            gql::TypeRef::To { name } => Self(name),
        }
    }
}

struct TypeRefIndex(HashMap<String, ts::TypeRef>);

impl TryFrom<(&TypeRefIndex, gql::NonNullTypeRef)> for ts::RefContainer<ts::TypeRef> {
    type Error = Report;

    fn try_from(
        (type_ref_index, value): (&TypeRefIndex, gql::NonNullTypeRef),
    ) -> std::result::Result<Self, Self::Error> {
        let type_container = match value {
            gql::NonNullTypeRef::Container(gql::NonNullTypeRefContainer::List { of_type }) => {
                Self::List(Box::new((type_ref_index, *of_type).try_into()?))
            }
            gql::NonNullTypeRef::To { name } => Self::Ref(
                type_ref_index
                    .0
                    .get(&name)
                    .cloned()
                    .ok_or_else(|| eyre!("Couldn't find TypeRef"))?,
            ),
        };

        Ok(type_container)
    }
}

impl TryFrom<(&TypeRefIndex, gql::TypeRef)> for ts::RefContainer<ts::TypeRef> {
    type Error = Report;

    fn try_from(
        (type_ref_index, value): (&TypeRefIndex, gql::TypeRef),
    ) -> std::result::Result<Self, Self::Error> {
        let type_container = match value {
            gql::TypeRef::Container(gql::TypeRefContainer::List { of_type }) => {
                Self::Nullable(ts::NullableRefContainer::List(Box::new(Self::List(
                    Box::new((type_ref_index, *of_type).try_into()?),
                ))))
            }
            gql::TypeRef::Container(gql::TypeRefContainer::NonNull { of_type }) => {
                (type_ref_index, *of_type).try_into()?
            }
            gql::TypeRef::To { name } => Self::Nullable(ts::NullableRefContainer::Ref(
                type_ref_index
                    .0
                    .get(&name)
                    .cloned()
                    .ok_or_else(|| eyre!("Couldn't find TypeRef"))?,
            )),
        };

        Ok(type_container)
    }
}

impl TryFrom<(&TypeRefIndex, gql::Field)> for ts::Field {
    type Error = Report;

    fn try_from(
        (type_ref_index, value): (&TypeRefIndex, gql::Field),
    ) -> std::result::Result<Self, Self::Error> {
        let gql::Field {
            name,
            description,
            args: _,
            of_type,
            is_deprecated,
            deprecation_reason,
        } = value;

        Ok(Self {
            name,
            doc_comment: ts::DocComment::maybe_new(is_deprecated, deprecation_reason, description),
            of_type: (type_ref_index, of_type).try_into()?,
        })
    }
}

impl TryFrom<(&TypeRefIndex, gql::named_type::Object)> for ts::Object {
    type Error = Report;

    fn try_from(
        (type_ref_index, value): (&TypeRefIndex, gql::named_type::Object),
    ) -> Result<Self, Self::Error> {
        let gql::named_type::Object {
            name,
            description,
            fields,
            interfaces,
        } = value;

        Ok(Self {
            name,
            doc_comment: description.map(ts::DocComment),
            interfaces: interfaces.into_iter().map(Into::into).collect(),
            fields: fields
                .into_iter()
                .map(|field| (type_ref_index, field).try_into())
                .collect::<Result<_>>()?,
        })
    }
}

impl From<(gql::named_type::Scalar, &TypescriptOptions)> for ts::Scalar {
    fn from((value, options): (gql::named_type::Scalar, &TypescriptOptions)) -> Self {
        let gql::named_type::Scalar { name, description } = value;

        Self {
            name: name.clone(),
            doc_comment: description.map(ts::DocComment),
            of_type: ts::ScalarType::new(name, options),
        }
    }
}
