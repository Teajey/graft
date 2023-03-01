use graphql_parser::query as gp;

use crate::graphql::query as ac;

use super::ObjectField;

impl From<gp::Type<'_, String>> for ac::Type {
    fn from(t: gp::Type<'_, String>) -> Self {
        match t {
            gp::Type::NamedType(name) => ac::Type::Named {
                name: ac::Name(name),
            },
            gp::Type::ListType(value) => ac::Type::List {
                value: Box::new((*value).into()),
            },
            gp::Type::NonNullType(value) => ac::Type::NonNull {
                value: Box::new((*value).into()),
            },
        }
    }
}

impl From<gp::Value<'_, String>> for ac::Value {
    fn from(value: gp::Value<'_, String>) -> Self {
        match value {
            gp::Value::Variable(var) => ac::Value::Variable {
                name: ac::Name(var),
            },
            gp::Value::Int(i) => ac::Value::Int {
                // TODO: Should just be an i64
                value: i.as_i64().expect("must be i64").to_string(),
            },
            gp::Value::Float(value) => ac::Value::Float { value },
            gp::Value::String(value) => ac::Value::String { value, block: true },
            gp::Value::Boolean(value) => ac::Value::Boolean { value },
            gp::Value::Null => ac::Value::Null,
            gp::Value::Enum(value) => ac::Value::Enum { value },
            gp::Value::List(list) => ac::Value::List {
                values: list.into_iter().map(ac::Value::from).collect(),
            },
            gp::Value::Object(map) => {
                let fields = map
                    .into_iter()
                    .map(|(k, v)| ObjectField {
                        kind: Some(ac::tag::ObjectField::T),
                        name: ac::Name(k),
                        value: v.into(),
                    })
                    .collect();
                ac::Value::Object { fields }
            }
        }
    }
}

impl From<gp::VariableDefinition<'_, String>> for ac::VariableDefinition {
    fn from(
        gp::VariableDefinition {
            position: _,
            name,
            var_type,
            default_value,
        }: gp::VariableDefinition<'_, String>,
    ) -> Self {
        Self {
            kind: Some(ac::tag::VariableDefinition::T),
            variable: ac::Variable {
                kind: ac::tag::Variable::T,
                name: ac::Name(name),
            },
            of_type: var_type.into(),
            default_value: default_value.map(Into::into),
            directives: vec![],
        }
    }
}

impl From<gp::Directive<'_, String>> for ac::Directive {
    fn from(
        gp::Directive {
            position: _,
            name,
            arguments,
        }: gp::Directive<'_, String>,
    ) -> Self {
        Self {
            kind: Some(ac::tag::Directive::T),
            name: ac::Name(name),
            arguments: arguments
                .into_iter()
                .map(|(name, value)| ac::Argument {
                    kind: Some(ac::tag::Argument::T),
                    name: ac::Name(name),
                    value: value.into(),
                })
                .collect(),
        }
    }
}

impl From<gp::Selection<'_, String>> for ac::Selection {
    fn from(value: gp::Selection<'_, String>) -> Self {
        match value {
            gp::Selection::Field(gp::Field {
                position: _,
                alias,
                name,
                arguments,
                directives,
                selection_set,
            }) => ac::Selection::Field {
                alias: alias.map(ac::Name),
                name: ac::Name(name),
                arguments: arguments
                    .into_iter()
                    .map(|(name, value)| ac::Argument {
                        kind: Some(ac::tag::Argument::T),
                        name: ac::Name(name),
                        value: value.into(),
                    })
                    .collect(),
                directives: directives.into_iter().map(Into::into).collect(),
                // FIXME: Surely not every field is going to have a selection set?
                selection_set: Some(selection_set.into()),
            },
            gp::Selection::FragmentSpread(gp::FragmentSpread {
                position: _,
                fragment_name,
                directives,
            }) => ac::Selection::FragmentSpread {
                name: ac::Name(fragment_name),
                directives: directives.into_iter().map(Into::into).collect(),
            },
            gp::Selection::InlineFragment(gp::InlineFragment {
                position: _,
                type_condition,
                directives,
                selection_set,
            }) => ac::Selection::InlineFragment {
                type_condition: type_condition.map(|gp::TypeCondition::On(name)| ac::NamedType {
                    kind: Some(ac::tag::NamedType::T),
                    name: ac::Name(name),
                }),
                directives: directives.into_iter().map(Into::into).collect(),
                selection_set: selection_set.into(),
            },
        }
    }
}

impl From<gp::SelectionSet<'_, String>> for ac::SelectionSet {
    fn from(gp::SelectionSet { span: _, items }: gp::SelectionSet<'_, String>) -> Self {
        Self {
            kind: Some(ac::tag::SelectionSet::T),
            selections: items.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<gp::OperationDefinition<'_, String>> for ac::Operation {
    fn from(op: gp::OperationDefinition<'_, String>) -> Self {
        match op {
            gp::OperationDefinition::SelectionSet(selection_set) => {
                ac::Operation::SelectionSet(selection_set.into())
            }
            gp::OperationDefinition::Query(gp::Query {
                position: _,
                name,
                variable_definitions,
                directives,
                selection_set,
            }) => ac::Operation::Query {
                name: name.map(ac::Name),
                variable_definitions: variable_definitions.into_iter().map(Into::into).collect(),
                directives: directives.into_iter().map(Into::into).collect(),
                selection_set: selection_set.into(),
            },
            gp::OperationDefinition::Mutation(gp::Mutation {
                position: _,
                name,
                variable_definitions,
                directives,
                selection_set,
            }) => ac::Operation::Mutation {
                name: name.map(ac::Name),
                variable_definitions: variable_definitions.into_iter().map(Into::into).collect(),
                directives: directives.into_iter().map(Into::into).collect(),
                selection_set: selection_set.into(),
            },
            gp::OperationDefinition::Subscription(gp::Subscription {
                position: _,
                name,
                variable_definitions,
                directives,
                selection_set,
            }) => ac::Operation::Subscription {
                name: name.map(ac::Name),
                variable_definitions: variable_definitions.into_iter().map(Into::into).collect(),
                directives: directives.into_iter().map(Into::into).collect(),
                selection_set: selection_set.into(),
            },
        }
    }
}

impl From<gp::Definition<'_, String>> for ac::Definition {
    fn from(def: gp::Definition<'_, String>) -> Self {
        match def {
            gp::Definition::Operation(op) => ac::Definition::Operation(op.into()),
            gp::Definition::Fragment(gp::FragmentDefinition {
                position: _,
                name,
                type_condition: gp::TypeCondition::On(tc_name),
                directives,
                selection_set,
            }) => ac::Definition::Fragment {
                name: ac::Name(name),
                type_condition: ac::NamedType {
                    kind: Some(ac::tag::NamedType::T),
                    name: ac::Name(tc_name),
                },
                directives: directives.into_iter().map(Into::into).collect(),
                selection_set: selection_set.into(),
            },
        }
    }
}
