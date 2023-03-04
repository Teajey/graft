use std::collections::HashMap;

use eyre::{eyre, Report, Result};

use crate::{
    graphql::{query, schema},
    typescript::{ts, TypescriptOptions},
};

struct TypesIndex<'t> {
    named: HashMap<String, ts::NamedType<'t>>,
    fielded: HashMap<String, ts::FieldedType<'t>>,
}

type FragmentsIndex<'t> = HashMap<String, ts::Fragment<'t>>;

pub struct Context<'t> {
    options: TypescriptOptions,
    types: TypesIndex<'t>,
    fragments: FragmentsIndex<'t>,
}

pub struct WithFieldedType<'t, T> {
    target: T,
    of_type: &'t ts::FieldedType<'t>,
}

impl<'t> ts::FieldedType<'t> {
    pub fn with<T>(&'t self, target: T) -> WithFieldedType<'t, T> {
        WithFieldedType {
            target,
            of_type: self,
        }
    }
}

pub struct WithType<'t, T> {
    target: T,
    of_type: &'t ts::Type<'t>,
}

impl<'t> ts::Type<'t> {
    pub fn with<T>(&'t self, target: T) -> WithType<'t, T> {
        WithType {
            target,
            of_type: self,
        }
    }
}

pub struct WithNamedType<'t, T> {
    target: T,
    of_type: &'t ts::NamedType<'t>,
}

impl<'t> ts::NamedType<'t> {
    pub fn with<T>(&'t self, target: T) -> WithNamedType<'t, T> {
        WithNamedType {
            target,
            of_type: self,
        }
    }
}

pub struct WithTypesIndex<'t, T> {
    target: T,
    types: &'t TypesIndex<'t>,
}

impl<'t> TypesIndex<'t> {
    pub fn with<T>(&'t self, target: T) -> WithTypesIndex<'t, T> {
        WithTypesIndex {
            target,
            types: self,
        }
    }
}

pub struct WithContext<'c, 't, T> {
    target: T,
    ctx: &'c Context<'t>,
}

impl<'c, 't> Context<'t> {
    pub fn with<T>(&'c self, target: T) -> WithContext<'c, 't, T> {
        WithContext { target, ctx: self }
    }
}

impl<'c, 't> TryFrom<WithContext<'c, 't, query::Operation>> for ts::Operation<'t> {
    type Error = Report;

    fn try_from(value: WithContext<'c, 't, query::Operation>) -> Result<Self> {
        todo!()
    }
}

#[derive(Clone)]
enum LeafType<'t> {
    Enum(&'t ts::Enum),
    Scalar(&'t ts::Scalar),
}

#[derive(Clone)]
enum TreeType<'t> {
    Trunk(ts::FieldedType<'t>),
    Leaf(LeafType<'t>),
}

impl<'t> ts::NamedType<'t> {
    fn as_tree_type(&'t self) -> TreeType<'t> {
        match self {
            ts::NamedType::Object(object) => TreeType::Trunk(ts::FieldedType::Object(object)),
            ts::NamedType::Interface(interface) => {
                TreeType::Trunk(ts::FieldedType::Interface(interface))
            }
            ts::NamedType::Union(u) => TreeType::Trunk(ts::FieldedType::Union(u)),
            ts::NamedType::Scalar(scalar) => TreeType::Leaf(LeafType::Scalar(scalar)),
            ts::NamedType::Enum(e) => TreeType::Leaf(LeafType::Enum(e)),
        }
    }
}

impl<'t> TryFrom<WithFieldedType<'t, (&'t ts::NamedType<'t>, Option<query::SelectionSet>)>>
    for ts::NamedSelectionType<'t>
{
    type Error = Report;

    fn try_from(
        value: WithFieldedType<'t, (&'t ts::NamedType<'t>, Option<query::SelectionSet>)>,
    ) -> Result<Self> {
        let WithFieldedType {
            target: (named, selection_set),
            of_type,
        } = value;

        let selection_value = match named.as_tree_type() {
            TreeType::Trunk(trunk) => {
                let Some(selection_set) = selection_set else {
                    return Err(eyre!("Didn't provide a selection set on a type with fields."));
                };
                Self::SelectionSet(trunk.with(selection_set).try_into()?)
            }
            TreeType::Leaf(_) => {
                if selection_set.is_some() {
                    return Err(eyre!("Selection set on a non-selectable type"));
                }
                Self::On(of_type)
            }
        };

        Ok(selection_value)
    }
}

impl<'t> TryFrom<WithFieldedType<'t, (&ts::Field<'t>, Option<query::SelectionSet>)>>
    for ts::SelectionType<'t>
{
    type Error = Report;

    fn try_from(
        value: WithFieldedType<'t, (&ts::Field<'t>, Option<query::SelectionSet>)>,
    ) -> Result<Self> {
        let WithFieldedType {
            target: (on_type, selection_set),
            of_type,
        } = value;

        let selection_value = match on_type.of_type {
            ts::Type::Named(named) => {
                ts::SelectionType::Named(of_type.with((named, selection_set)).try_into()?)
            }
            ts::Type::List(_) => todo!(),
            ts::Type::NonNull(_) => todo!(),
        };

        Ok(selection_value)
    }
}

impl<'t> TryFrom<WithFieldedType<'t, query::Field>> for ts::Selection<'t> {
    type Error = Report;

    fn try_from(value: WithFieldedType<'t, query::Field>) -> Result<Self> {
        let WithFieldedType {
            target: field,
            of_type,
        } = value;

        let alias = field.alias.map(|name| name.0);
        let selection_set = field.selection_set;

        let field = of_type
            .get_field(&field.name.0)
            .ok_or_else(|| eyre!("Selection on a non-existent field"))?;

        let name = ts::SelectionName {
            name: &field.name,
            alias,
        };

        Ok(ts::Selection {
            name,
            of_type: of_type.with((field, selection_set)).try_into()?,
        })
    }
}

impl<'t> TryFrom<WithFieldedType<'t, query::SelectionSet>> for ts::SelectionSet<'t> {
    type Error = Report;

    fn try_from(value: WithFieldedType<'t, query::SelectionSet>) -> Result<Self> {
        let WithFieldedType {
            target: selection_set,
            of_type,
        } = value;

        let selection_set = selection_set
            .selections
            .into_iter()
            .map(|s| match s {
                query::Selection::Field(field) => of_type.with(field).try_into(),
                query::Selection::InlineFragment(_) => todo!(),
                query::Selection::FragmentSpread(_) => todo!(),
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(ts::SelectionSet(selection_set))
    }
}

impl<'t> TryFrom<WithTypesIndex<'t, query::Fragment>> for ts::Fragment<'t> {
    type Error = Report;

    fn try_from(value: WithTypesIndex<'t, query::Fragment>) -> Result<Self> {
        let WithTypesIndex {
            target: fragment,
            types,
        } = value;

        let type_condition = types
            .fielded
            .get(&fragment.type_condition.name.0)
            .ok_or_else(|| eyre!("Fragment on invalid type"))?;

        Ok(Self {
            name: fragment.name.0.clone(),
            selection_set: type_condition
                .with(fragment.selection_set.clone())
                .try_into()?,
            type_condition,
            doc: fragment,
        })
    }
}

impl<'t> From<schema::NamedType> for ts::NamedType<'t> {
    fn from(value: schema::NamedType) -> Self {
        todo!()
    }
}
