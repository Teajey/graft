#![warn(unused)]

use std::collections::HashMap;

use eyre::{eyre, Report, Result};

use crate::{graphql::query, typescript::ts};

struct TypesIndex<'t>(HashMap<String, ts::NamedType<'t>>);

impl<'t> TypesIndex<'t> {
    fn get_fielded(&self, k: &str) -> Option<&ts::FieldedType> {
        match self.0.get(k) {
            Some(ts::NamedType::Fielded(f)) => Some(f),
            _ => None,
        }
    }
}

struct With<'t, S, T> {
    target: T,
    source: &'t S,
}

impl<'t> ts::Field<'t> {
    fn with<T>(&'t self, target: T) -> With<'t, Self, T> {
        With {
            target,
            source: self,
        }
    }
}

impl<'t> ts::NamedType<'t> {
    fn with<T>(&'t self, target: T) -> With<'t, Self, T> {
        With {
            target,
            source: self,
        }
    }
}

impl<'t> ts::FieldedType<'t> {
    fn with<T>(&'t self, target: T) -> With<'t, Self, T> {
        With {
            target,
            source: self,
        }
    }
}

impl<'t> TypesIndex<'t> {
    fn with<T>(&'t self, target: T) -> With<'t, Self, T> {
        With {
            target,
            source: self,
        }
    }
}

impl<'t> TryFrom<With<'t, ts::NamedType<'t>, Option<query::SelectionSet>>>
    for ts::NamedSelectionType<'t>
{
    type Error = Report;

    fn try_from(
        value: With<'t, ts::NamedType<'t>, Option<query::SelectionSet>>,
    ) -> std::result::Result<Self, Self::Error> {
        let With {
            target: selection_set,
            source: named,
        } = value;

        let named_selection_type = match named {
            ts::NamedType::Fielded(fielded) => {
                let Some(selection_set) = selection_set else {
                    return Err(eyre!("A fielded type must have a subselection"));
                };
                ts::NamedSelectionType::SelectionSet(fielded.with(selection_set).try_into()?)
            }
            ts::NamedType::Leaf(leaf) => ts::NamedSelectionType::Leaf(leaf),
        };

        Ok(named_selection_type)
    }
}

impl<'t> TryFrom<With<'t, ts::Field<'t>, Option<query::SelectionSet>>> for ts::SelectionType<'t> {
    type Error = Report;

    fn try_from(value: With<'t, ts::Field<'t>, Option<query::SelectionSet>>) -> Result<Self> {
        let With {
            target: selection_set,
            source: field,
        } = value;

        let selection_type = match field.of_type {
            ts::Type::Named(named) => {
                ts::SelectionType::Named(named.with(selection_set).try_into()?)
            }
            ts::Type::List(_) => todo!(),
            ts::Type::NonNull(_) => todo!(),
        };

        Ok(selection_type)
    }
}

impl<'t> TryFrom<With<'t, ts::FieldedType<'t>, query::Field>> for ts::Selection<'t> {
    type Error = Report;

    fn try_from(value: With<'t, ts::FieldedType<'t>, query::Field>) -> Result<Self> {
        let With {
            target: field,
            source,
        } = value;

        let alias = field.alias.map(|name| name.0);
        let selection_set = field.selection_set;

        let field = source
            .get_field(&field.name.0)
            .ok_or_else(|| eyre!("Selection on a non-existent field"))?;

        let name = ts::SelectionName {
            name: &field.name,
            alias,
        };

        Ok(ts::Selection {
            name,
            of_type: field.with(selection_set).try_into()?,
        })
    }
}

impl<'t> TryFrom<With<'t, ts::FieldedType<'t>, query::SelectionSet>> for ts::SelectionSet<'t> {
    type Error = Report;

    fn try_from(value: With<'t, ts::FieldedType, query::SelectionSet>) -> Result<Self> {
        let With {
            target: selection_set,
            source,
        } = value;

        let selection_set = selection_set
            .selections
            .into_iter()
            .map(|s| match s {
                query::Selection::Field(field) => source.with(field).try_into(),
                query::Selection::InlineFragment(_) => todo!(),
                query::Selection::FragmentSpread(_) => todo!(),
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(ts::SelectionSet(selection_set))
    }
}

impl<'t> TryFrom<With<'t, TypesIndex<'t>, query::Fragment>> for ts::Fragment<'t> {
    type Error = Report;

    fn try_from(value: With<'t, TypesIndex<'t>, query::Fragment>) -> Result<Self> {
        let With {
            target: fragment,
            source: types,
        } = value;

        let type_condition = types
            .get_fielded(&fragment.type_condition.name.0)
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
