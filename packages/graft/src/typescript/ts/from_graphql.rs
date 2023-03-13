use std::collections::HashMap;

use eyre::{eyre, Report, Result};

use crate::{
    graphql::query,
    typescript::ts::{self, Fielded, PossibleTypes},
};

struct TypesIndex<'t>(HashMap<String, ts::NamedType<'t>>);

impl<'t> TypesIndex<'t> {
    fn get_fielded(&self, k: &str) -> Option<&ts::FieldedType> {
        match self.0.get(k) {
            Some(ts::NamedType::Fielded(f)) => Some(f),
            _ => None,
        }
    }

    fn get_object(&self, k: &str) -> Option<&ts::Object> {
        match self.0.get(k) {
            Some(ts::NamedType::Fielded(ts::FieldedType::Object(o))) => Some(o),
            _ => None,
        }
    }
}

struct FragmentsIndex<'t>(HashMap<String, ts::Fragment<'t>>);

struct InputsIndex<'t>(HashMap<String, ts::InputType<'t>>);

struct Index<'t> {
    types: TypesIndex<'t>,
    fragments: FragmentsIndex<'t>,
    inputs: InputsIndex<'t>,
    query: ts::Object<'t>,
    mutation: Option<ts::Object<'t>>,
    subscription: Option<ts::Object<'t>>,
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

impl<'t> TryFrom<WithIndex<'t, (&'t ts::NullableType<'t>, Option<query::SelectionSet>)>>
    for ts::NullableSelectionType<'t>
{
    type Error = Report;

    fn try_from(
        value: WithIndex<'t, (&'t ts::NullableType<'t>, Option<query::SelectionSet>)>,
    ) -> Result<Self> {
        let WithIndex {
            index,
            bundle: (of_type, selection_set),
        } = value;

        let non_null_selection_type = match of_type {
            ts::NullableType::Named(named) => {
                Self::Named(index.with((*named, selection_set)).try_into()?)
            }
            ts::NullableType::List(list) => {
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
            ts::Type::Nullable(non_null) => {
                Self::Nullable(index.with((non_null, selection_set)).try_into()?)
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
            ts::Type::Nullable(non_null) => {
                Self::Nullable(index.with((non_null, selection_set)).try_into()?)
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

        let spreaded_object = match fielded {
            ts::FieldedType::Object(o) => {
                if let Some(tc) = inline_fragment.type_condition {
                    return Err(eyre!(
                        "Cannot spread an object ({tc:?}) into another object"
                    ));
                }
                o
            }
            ts::FieldedType::Union(u) => {
                let tc = inline_fragment
                    .type_condition
                    .map(|tc| tc.name.0)
                    .ok_or_else(|| {
                        eyre!(
                            "Must provide an object as the target of an inline fragment on a union"
                        )
                    })?;
                u.get_possible_type(&tc).ok_or_else(|| {
                    eyre!("Inline fragment targetting object that does not implement this union")
                })?
            }
            ts::FieldedType::Interface(i) => {
                let tc = inline_fragment.type_condition.map(|tc| tc.name.0).ok_or_else(|| eyre!("Must provide an object as the target of an inline fragment on an interface"))?;
                i.get_possible_type(&tc).ok_or_else(|| {
                    eyre!(
                        "Inline fragment targetting object that does not implement this interface"
                    )
                })?
            }
        };

        let fielded = index
            .types
            .get_fielded(&spreaded_object.name)
            .expect("Something is very wrong if this object doesn't exist");

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

impl<'t> TryFrom<WithIndex<'t, query::NonNullType>> for ts::Type<'t, ts::InputType<'t>> {
    type Error = Report;

    fn try_from(
        value: WithIndex<'t, query::NonNullType>,
    ) -> std::result::Result<Self, Self::Error> {
        let WithIndex {
            index,
            bundle: of_type,
        } = value;

        let of_type = match of_type {
            query::NonNullType::Named { name } => {
                let input_type = index
                    .inputs
                    .0
                    .get(&name.0)
                    .ok_or_else(|| eyre!("Could not find input type: {}", name.0))?;
                ts::Type::Named(input_type)
            }
            query::NonNullType::List { value } => {
                ts::Type::List(Box::new(index.with(*value).try_into()?))
            }
        };

        Ok(of_type)
    }
}

impl<'t> TryFrom<WithIndex<'t, query::Type>> for ts::Type<'t, ts::InputType<'t>> {
    type Error = Report;

    fn try_from(value: WithIndex<'t, query::Type>) -> std::result::Result<Self, Self::Error> {
        let WithIndex {
            index,
            bundle: of_type,
        } = value;

        let of_type = match of_type {
            query::Type::Named { name } => {
                let input_type = index
                    .inputs
                    .0
                    .get(&name.0)
                    .ok_or_else(|| eyre!("Could not find input type: {}", name.0))?;
                ts::Type::Nullable(ts::NullableType::Named(input_type))
            }
            query::Type::NonNull { value } => index.with(value).try_into()?,
            query::Type::List { value } => ts::Type::Nullable(ts::NullableType::List(Box::new(
                index.with(*value).try_into()?,
            ))),
        };

        Ok(of_type)
    }
}

impl<'t> TryFrom<WithIndex<'t, query::VariableDefinition>> for ts::Argument<'t> {
    type Error = Report;

    fn try_from(
        value: WithIndex<'t, query::VariableDefinition>,
    ) -> std::result::Result<Self, Self::Error> {
        let WithIndex {
            index,
            bundle:
                query::VariableDefinition {
                    variable,
                    of_type,
                    default_value,
                    directives,
                    ..
                },
        } = value;

        Ok(ts::Argument {
            name: variable.name.0,
            description: None,
            of_type: index.with(of_type).try_into()?,
        })
    }
}

impl<'t> TryFrom<WithIndex<'t, query::NamedOperation>> for ts::Operation<'t> {
    type Error = Report;

    fn try_from(
        value: WithIndex<'t, query::NamedOperation>,
    ) -> std::result::Result<Self, Self::Error> {
        let WithIndex {
            index,
            bundle: named,
        } = value;

        let doc = named.clone();

        let query::NamedOperation {
            operation,
            name,
            variable_definitions: query::VariableDefinitions(variable_definitions),
            directives,
            selection_set,
        } = named;

        let name =
            name.ok_or_else(|| eyre!("Typescripting anonymous operations is not supported"))?;

        let (operation_object, of_type) = match operation {
            query::OperationType::Query => {
                let query = index
                    .types
                    .get_fielded(&index.query.name)
                    .expect("Query must exist, it was already in index");
                (query, ts::OperationType::Query)
            }
            query::OperationType::Mutation => {
                let name = index
                    .mutation
                    .as_ref()
                    .map(|m| m.name.as_str())
                    .ok_or_else(|| eyre!("Tried to declare a mutation without a mutation root"))?;
                let mutation = index
                    .types
                    .get_fielded(name)
                    .expect("Mutation must exist, it was already in index");
                (mutation, ts::OperationType::Mutation)
            }
            query::OperationType::Subscription => {
                let name = index
                    .subscription
                    .as_ref()
                    .map(|m| m.name.as_str())
                    .ok_or_else(|| {
                        eyre!("Tried to declare a subscription without a subscription root")
                    })?;
                let subscription = index
                    .types
                    .get_fielded(name)
                    .expect("Subscription must exist, it was already in index");
                (subscription, ts::OperationType::Subscription)
            }
        };

        Ok(ts::Operation {
            of_type,
            name: name.0,
            arguments: ts::Arguments(
                variable_definitions
                    .into_iter()
                    .map(|vd| index.with(vd).try_into())
                    .collect::<Result<Vec<_>>>()?,
            ),
            selection_set: index.with((operation_object, selection_set)).try_into()?,
            doc,
        })
    }
}

impl<'t> TryFrom<WithIndex<'t, query::NonNullType>> for ts::Type<'t> {
    type Error = Report;

    fn try_from(
        value: WithIndex<'t, query::NonNullType>,
    ) -> std::result::Result<Self, Self::Error> {
        let WithIndex {
            index,
            bundle: of_type,
        } = value;

        let of_type = match of_type {
            query::NonNullType::Named { name } => {
                let named = index
                    .types
                    .0
                    .get(&name.0)
                    .ok_or_else(|| eyre!("Tried to ts a non-existent type"))?;
                ts::Type::Named(named)
            }
            query::NonNullType::List { value } => {
                ts::Type::List(Box::new(index.with(*value).try_into()?))
            }
        };

        Ok(of_type)
    }
}

impl<'t> TryFrom<WithIndex<'t, query::Type>> for ts::Type<'t> {
    type Error = Report;

    fn try_from(value: WithIndex<'t, query::Type>) -> std::result::Result<Self, Self::Error> {
        let WithIndex {
            index,
            bundle: of_type,
        } = value;

        let of_type = match of_type {
            query::Type::Named { name } => {
                let named = index
                    .types
                    .0
                    .get(&name.0)
                    .ok_or_else(|| eyre!("Tried to ts a non-existent type"))?;
                ts::Type::Nullable(ts::NullableType::Named(named))
            }
            query::Type::NonNull { value } => index.with(value).try_into()?,
            query::Type::List { value } => ts::Type::Nullable(ts::NullableType::List(Box::new(
                index.with(*value).try_into()?,
            ))),
        };

        Ok(of_type)
    }
}

#[cfg(test)]
mod tests {
    use map_macro::map;
    use pretty_assertions::assert_eq;

    use crate::graphql::query;

    use super::{ts, FragmentsIndex, Index, InputsIndex, TypesIndex};

    fn nullable() {
        let query = ts::Object {
            name: "Query".to_string(),
            description: None,
            interfaces: vec![],
            fields: vec![],
        };

        let string = ts::NamedType::Leaf(ts::LeafType::Scalar(ts::Scalar {
            name: "String".to_string(),
            description: None,
        }));

        let index = Index {
            types: TypesIndex(map! {
                "String".to_string() => string,
            }),
            fragments: FragmentsIndex(map! {}),
            inputs: InputsIndex(map! {}),
            query,
            mutation: None,
            subscription: None,
        };

        let type_ref = index.types.0.get("String").expect("String type must exist");

        let weird_gql_type = query::Type::NonNull {
            value: query::NonNullType::List {
                value: Box::new(query::Type::NonNull {
                    value: query::NonNullType::Named {
                        name: query::Name("String".to_string()),
                    },
                }),
            },
        };

        let expected_ts_type = ts::Type::List(Box::new(ts::Type::List(Box::new(ts::Type::Named(
            type_ref,
        )))));

        let actual_ts_type: ts::Type = index
            .with(weird_gql_type)
            .try_into()
            .expect("must transform");

        // assert_eq!(expected_ts_type, actual_ts_type);
    }
}
