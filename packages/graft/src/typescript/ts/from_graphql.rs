#![warn(unused)]

use std::collections::HashMap;

use eyre::{eyre, Report, Result};

use crate::{graphql::query, typescript::ts};

struct TypesIndex<'t>(HashMap<String, ts::NamedType<'t>>);

struct FragmentsIndex<'t>(HashMap<String, ts::Fragment<'t>>);

struct Index<'t> {
    types: TypesIndex<'t>,
    fragments: FragmentsIndex<'t>,
}

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

impl<'t> ts::Type<'t> {
    fn with<T>(&'t self, target: T) -> With<'t, Self, T> {
        With {
            target,
            source: self,
        }
    }
}

impl<'t> ts::NonNullType<'t> {
    fn with<T>(&'t self, target: T) -> With<'t, Self, T> {
        With {
            target,
            source: self,
        }
    }
}

impl<'t> FragmentsIndex<'t> {
    fn with<T>(&'t self, target: T) -> With<'t, Self, T> {
        With {
            target,
            source: self,
        }
    }
}

impl<'t> TryFrom<With<'t, ts::NamedType<'t>, (&'t Index<'t>, Option<query::SelectionSet>)>>
    for ts::NamedSelectionType<'t>
{
    type Error = Report;

    fn try_from(
        value: With<'t, ts::NamedType<'t>, (&'t Index<'t>, Option<query::SelectionSet>)>,
    ) -> std::result::Result<Self, Self::Error> {
        let With {
            target: (index, selection_set),
            source: named,
        } = value;

        let named_selection_type = match named {
            ts::NamedType::Fielded(fielded) => {
                let Some(selection_set) = selection_set else {
                    return Err(eyre!("A fielded type must have a subselection"));
                };
                ts::NamedSelectionType::SelectionSet(
                    fielded.with((index, selection_set)).try_into()?,
                )
            }
            ts::NamedType::Leaf(leaf) => ts::NamedSelectionType::Leaf(leaf),
        };

        Ok(named_selection_type)
    }
}

impl<'t> TryFrom<With<'t, ts::NonNullType<'t>, (&'t Index<'t>, Option<query::SelectionSet>)>>
    for ts::NonNullSelectionType<'t>
{
    type Error = Report;

    fn try_from(
        value: With<'t, ts::NonNullType<'t>, (&'t Index<'t>, Option<query::SelectionSet>)>,
    ) -> Result<Self> {
        let With {
            target: (index, selection_set),
            source: of_type,
        } = value;

        let non_null_selection_type = match of_type {
            ts::NonNullType::Named(named) => {
                Self::Named(named.with((index, selection_set)).try_into()?)
            }
            ts::NonNullType::List(list) => {
                Self::List(Box::new(list.with((index, selection_set)).try_into()?))
            }
        };

        Ok(non_null_selection_type)
    }
}

impl<'t> TryFrom<With<'t, ts::Type<'t>, (&'t Index<'t>, Option<query::SelectionSet>)>>
    for ts::ListSelectionType<'t>
{
    type Error = Report;

    fn try_from(
        value: With<'t, ts::Type<'t>, (&'t Index<'t>, Option<query::SelectionSet>)>,
    ) -> Result<Self> {
        let With {
            target: (index, selection_set),
            source: of_type,
        } = value;

        let list_selection_type = match of_type {
            ts::Type::Named(named) => Self::Named(named.with((index, selection_set)).try_into()?),
            ts::Type::List(list) => {
                Self::List(Box::new(list.with((index, selection_set)).try_into()?))
            }
            ts::Type::NonNull(non_null) => {
                Self::NonNull(non_null.with((index, selection_set)).try_into()?)
            }
        };

        Ok(list_selection_type)
    }
}

impl<'t> TryFrom<With<'t, ts::Type<'t>, (&'t Index<'t>, Option<query::SelectionSet>)>>
    for ts::SelectionType<'t>
{
    type Error = Report;

    fn try_from(
        value: With<'t, ts::Type<'t>, (&'t Index<'t>, Option<query::SelectionSet>)>,
    ) -> Result<Self> {
        let With {
            target: (index, selection_set),
            source: of_type,
        } = value;

        let selection_type = match of_type {
            ts::Type::Named(named) => Self::Named(named.with((index, selection_set)).try_into()?),
            ts::Type::List(list) => Self::List(list.with((index, selection_set)).try_into()?),
            ts::Type::NonNull(non_null) => {
                Self::NonNull(non_null.with((index, selection_set)).try_into()?)
            }
        };

        Ok(selection_type)
    }
}

impl<'t> TryFrom<With<'t, ts::FieldedType<'t>, (&'t Index<'t>, query::Field)>>
    for ts::FieldSelection<'t>
{
    type Error = Report;

    fn try_from(
        value: With<'t, ts::FieldedType<'t>, (&'t Index<'t>, query::Field)>,
    ) -> Result<Self> {
        let With {
            target: (index, field),
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

        Ok(Self {
            name,
            of_type: field.of_type.with((index, selection_set)).try_into()?,
        })
    }
}

impl<'t> TryFrom<With<'t, FragmentsIndex<'t>, query::FragmentSpread>>
    for ts::FragmentSelection<'t>
{
    type Error = Report;

    fn try_from(
        value: With<'t, FragmentsIndex<'t>, query::FragmentSpread>,
    ) -> std::result::Result<Self, Self::Error> {
        let With {
            target:
                query::FragmentSpread {
                    name,
                    directives: _,
                },
            source: fragments,
        } = value;

        let fragment = fragments
            .0
            .get(&name.0)
            .ok_or_else(|| eyre!("Fragment spread with non-existent fragment"))?;

        Ok(ts::FragmentSelection(fragment.selection_set.clone()))
    }
}

impl<'t> TryFrom<With<'t, ts::FieldedType<'t>, (&'t Index<'t>, query::InlineFragment)>>
    for ts::FragmentSelection<'t>
{
    type Error = Report;

    fn try_from(
        value: With<'t, ts::FieldedType<'t>, (&'t Index<'t>, query::InlineFragment)>,
    ) -> std::result::Result<Self, Self::Error> {
        let With {
            target: (index, inline_fragment),
            source: fielded,
        } = value;

        let fielded = inline_fragment
            .type_condition
            .map(|tc| fielded.get_possible_type(&tc.name.0))
            .transpose()?
            .flatten()
            .and_then(|object| index.types.get_fielded(&object.name))
            .unwrap_or(fielded);

        Ok(ts::FragmentSelection(
            fielded
                .with((index, inline_fragment.selection_set))
                .try_into()?,
        ))
    }
}

impl<'t> TryFrom<With<'t, ts::FieldedType<'t>, (&'t Index<'t>, query::Selection)>>
    for ts::Selection<'t>
{
    type Error = Report;

    fn try_from(
        value: With<'t, ts::FieldedType<'t>, (&'t Index<'t>, query::Selection)>,
    ) -> Result<Self> {
        let With {
            target: (index, selection),
            source: fielded,
        } = value;

        let selection = match selection {
            query::Selection::Field(field) => Self::Field(fielded.with((index, field)).try_into()?),
            query::Selection::InlineFragment(inline_fragment) => {
                Self::Fragment(fielded.with((index, inline_fragment)).try_into()?)
            }
            query::Selection::FragmentSpread(fragment_spread) => {
                Self::Fragment(index.fragments.with(fragment_spread).try_into()?)
            }
        };

        Ok(selection)
    }
}

impl<'t> TryFrom<With<'t, ts::FieldedType<'t>, (&'t Index<'t>, query::SelectionSet)>>
    for ts::SelectionSet<'t>
{
    type Error = Report;

    fn try_from(
        value: With<'t, ts::FieldedType<'t>, (&'t Index<'t>, query::SelectionSet)>,
    ) -> Result<Self> {
        let With {
            target: (index, selection_set),
            source: fielded,
        } = value;

        Ok(ts::SelectionSet(
            selection_set
                .selections
                .into_iter()
                .map(|selection| fielded.with((index, selection)).try_into())
                .collect::<Result<_>>()?,
        ))
    }
}

impl<'t> TryFrom<With<'t, Index<'t>, query::Fragment>> for ts::Fragment<'t> {
    type Error = Report;

    fn try_from(value: With<'t, Index<'t>, query::Fragment>) -> Result<Self> {
        let With {
            target: fragment,
            source: index,
        } = value;

        let type_condition = index
            .types
            .get_fielded(&fragment.type_condition.name.0)
            .ok_or_else(|| eyre!("Fragment on invalid type"))?;

        Ok(Self {
            name: fragment.name.0.clone(),
            selection_set: type_condition
                .with((index, fragment.selection_set.clone()))
                .try_into()?,
            type_condition,
            doc: fragment,
        })
    }
}
