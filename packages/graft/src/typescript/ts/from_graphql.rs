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

struct FragmentsIndex<'t>(HashMap<String, ts::Fragment<'t>>);

struct Index<'t> {
    types: TypesIndex<'t>,
    fragments: FragmentsIndex<'t>,
}

struct WithIndex<'t, T> {
    index: &'t Index<'t>,
    bundle: T,
}

impl<'t> Index<'t> {
    fn with<T>(&'t self, bundle: T) -> WithIndex<'t, T> {
        WithIndex {
            index: self,
            bundle,
        }
    }
}

impl<'t> TryFrom<WithIndex<'t, (&'t ts::NamedType<'t>, Option<query::SelectionSet>)>>
    for ts::NamedSelectionType<'t>
{
    type Error = Report;

    fn try_from(
        value: WithIndex<'t, (&'t ts::NamedType<'t>, Option<query::SelectionSet>)>,
    ) -> std::result::Result<Self, Self::Error> {
        let WithIndex {
            index,
            bundle: (named, selection_set),
        } = value;

        let named_selection_type = match named {
            ts::NamedType::Fielded(fielded) => {
                let Some(selection_set) = selection_set else {
                    return Err(eyre!("A fielded type must have a subselection"));
                };
                ts::NamedSelectionType::SelectionSet(
                    index.with((fielded, selection_set)).try_into()?,
                )
            }
            ts::NamedType::Leaf(leaf) => ts::NamedSelectionType::Leaf(leaf),
        };

        Ok(named_selection_type)
    }
}

impl<'t> TryFrom<WithIndex<'t, (&'t ts::NonNullType<'t>, Option<query::SelectionSet>)>>
    for ts::NonNullSelectionType<'t>
{
    type Error = Report;

    fn try_from(
        value: WithIndex<'t, (&'t ts::NonNullType<'t>, Option<query::SelectionSet>)>,
    ) -> Result<Self> {
        let WithIndex {
            index,
            bundle: (of_type, selection_set),
        } = value;

        let non_null_selection_type = match of_type {
            ts::NonNullType::Named(named) => {
                Self::Named(index.with((*named, selection_set)).try_into()?)
            }
            ts::NonNullType::List(list) => {
                Self::List(Box::new(index.with((&**list, selection_set)).try_into()?))
            }
        };

        Ok(non_null_selection_type)
    }
}

impl<'t> TryFrom<WithIndex<'t, (&'t ts::Type<'t>, Option<query::SelectionSet>)>>
    for ts::ListSelectionType<'t>
{
    type Error = Report;

    fn try_from(
        value: WithIndex<'t, (&'t ts::Type<'t>, Option<query::SelectionSet>)>,
    ) -> Result<Self> {
        let WithIndex {
            index,
            bundle: (of_type, selection_set),
        } = value;

        let list_selection_type = match of_type {
            ts::Type::Named(named) => Self::Named(index.with((*named, selection_set)).try_into()?),
            ts::Type::List(list) => {
                Self::List(Box::new(index.with((&**list, selection_set)).try_into()?))
            }
            ts::Type::NonNull(non_null) => {
                Self::NonNull(index.with((non_null, selection_set)).try_into()?)
            }
        };

        Ok(list_selection_type)
    }
}

impl<'t> TryFrom<WithIndex<'t, (&'t ts::Type<'t>, Option<query::SelectionSet>)>>
    for ts::SelectionType<'t>
{
    type Error = Report;

    fn try_from(
        value: WithIndex<'t, (&'t ts::Type<'t>, Option<query::SelectionSet>)>,
    ) -> Result<Self> {
        let WithIndex {
            bundle: (of_type, selection_set),
            index,
        } = value;

        let selection_type = match of_type {
            ts::Type::Named(named) => Self::Named(index.with((*named, selection_set)).try_into()?),
            ts::Type::List(list) => Self::List(index.with((&**list, selection_set)).try_into()?),
            ts::Type::NonNull(non_null) => {
                Self::NonNull(index.with((non_null, selection_set)).try_into()?)
            }
        };

        Ok(selection_type)
    }
}

impl<'t> TryFrom<WithIndex<'t, (&'t ts::FieldedType<'t>, query::Field)>>
    for ts::FieldSelection<'t>
{
    type Error = Report;

    fn try_from(value: WithIndex<'t, (&'t ts::FieldedType<'t>, query::Field)>) -> Result<Self> {
        let WithIndex {
            bundle: (fielded, field),
            index,
        } = value;

        let alias = field.alias.map(|name| name.0);
        let selection_set = field.selection_set;

        let field = fielded
            .get_field(&field.name.0)
            .ok_or_else(|| eyre!("Selection on a non-existent field"))?;

        let name = ts::SelectionName {
            name: &field.name,
            alias,
        };

        Ok(Self {
            name,
            of_type: index.with((&field.of_type, selection_set)).try_into()?,
        })
    }
}

impl<'t> TryFrom<WithIndex<'t, query::FragmentSpread>> for ts::FragmentSelection<'t> {
    type Error = Report;

    fn try_from(
        value: WithIndex<'t, query::FragmentSpread>,
    ) -> std::result::Result<Self, Self::Error> {
        let WithIndex {
            bundle:
                query::FragmentSpread {
                    name,
                    directives: _,
                },
            index,
        } = value;

        let fragment = index
            .fragments
            .0
            .get(&name.0)
            .ok_or_else(|| eyre!("Fragment spread with non-existent fragment"))?;

        Ok(ts::FragmentSelection(fragment.selection_set.clone()))
    }
}

impl<'t> TryFrom<WithIndex<'t, (&'t ts::FieldedType<'t>, query::InlineFragment)>>
    for ts::FragmentSelection<'t>
{
    type Error = Report;

    fn try_from(
        value: WithIndex<'t, (&'t ts::FieldedType<'t>, query::InlineFragment)>,
    ) -> std::result::Result<Self, Self::Error> {
        let WithIndex {
            bundle: (fielded, inline_fragment),
            index,
        } = value;

        let fielded = inline_fragment
            .type_condition
            .map(|tc| fielded.get_possible_type(&tc.name.0))
            .transpose()?
            .flatten()
            .and_then(|object| index.types.get_fielded(&object.name))
            .unwrap_or(fielded);

        Ok(ts::FragmentSelection(
            index
                .with((fielded, inline_fragment.selection_set))
                .try_into()?,
        ))
    }
}

impl<'t> TryFrom<WithIndex<'t, (&'t ts::FieldedType<'t>, query::Selection)>> for ts::Selection<'t> {
    type Error = Report;

    fn try_from(value: WithIndex<'t, (&'t ts::FieldedType<'t>, query::Selection)>) -> Result<Self> {
        let WithIndex {
            bundle: (fielded, selection),
            index,
        } = value;

        let selection = match selection {
            query::Selection::Field(field) => Self::Field(index.with((fielded, field)).try_into()?),
            query::Selection::InlineFragment(inline_fragment) => {
                Self::Fragment(index.with((fielded, inline_fragment)).try_into()?)
            }
            query::Selection::FragmentSpread(fragment_spread) => {
                Self::Fragment(index.with(fragment_spread).try_into()?)
            }
        };

        Ok(selection)
    }
}

impl<'t> TryFrom<WithIndex<'t, (&'t ts::FieldedType<'t>, query::SelectionSet)>>
    for ts::SelectionSet<'t>
{
    type Error = Report;

    fn try_from(
        value: WithIndex<'t, (&'t ts::FieldedType<'t>, query::SelectionSet)>,
    ) -> Result<Self> {
        let WithIndex {
            bundle: (fielded, selection_set),
            index,
        } = value;

        Ok(ts::SelectionSet(
            selection_set
                .selections
                .into_iter()
                .map(|selection| index.with((fielded, selection)).try_into())
                .collect::<Result<_>>()?,
        ))
    }
}

impl<'t> TryFrom<WithIndex<'t, query::Fragment>> for ts::Fragment<'t> {
    type Error = Report;

    fn try_from(value: WithIndex<'t, query::Fragment>) -> Result<Self> {
        let WithIndex {
            bundle: fragment,
            index,
        } = value;

        let type_condition = index
            .types
            .get_fielded(&fragment.type_condition.name.0)
            .ok_or_else(|| eyre!("Fragment on invalid type"))?;

        Ok(Self {
            name: fragment.name.0.clone(),
            selection_set: index
                .with((type_condition, fragment.selection_set.clone()))
                .try_into()?,
            type_condition,
            doc: fragment,
        })
    }
}
