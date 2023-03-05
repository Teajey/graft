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

        todo!()
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
