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

impl<'t> TryFrom<WithFieldedType<'t, query::Type>> for ts::SelectionValueUtility<'t> {
    type Error = Report;

    fn try_from(value: WithFieldedType<'t, query::Type>) -> Result<Self> {
        let WithFieldedType {
            target: on_type,
            of_type,
        } = value;

        todo!()
    }
}

impl<'t> TryFrom<WithFieldedType<'t, query::Type>> for ts::SelectionValue<'t> {
    type Error = Report;

    fn try_from(value: WithFieldedType<'t, query::Type>) -> Result<Self> {
        let WithFieldedType {
            target: on_type,
            of_type,
        } = value;

        let selection_value = match on_type {
            query::Type::Named { name } => todo!(),
            query::Type::NonNull { value } => {
                ts::SelectionValue::Utility(of_type.with(*value).try_into()?)
            }
            query::Type::List { value } => todo!(),
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

        let field = of_type
            .get_field(&field.name.0)
            .ok_or_else(|| eyre!("Selection on a non-existent field"))?;

        let name = ts::SelectionName {
            name: &field.name,
            alias,
        };

        Ok(ts::Selection {
            name,
            value: of_type.with(field.of_type).try_into()?,
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
