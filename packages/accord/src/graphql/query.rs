mod from_ast;

use serde::Deserialize;

use super::kind::Kind;

#[derive(PartialEq, Debug, Kind)]
pub struct Name(String);

#[derive(Deserialize, Debug)]
pub struct Directive {
    name: Name,
    arguments: Vec<Argument>,
}

#[derive(Deserialize, Debug)]
pub struct ObjectField {
    name: Name,
    value: Value,
}

#[derive(Deserialize, Debug)]
pub struct Variable {
    name: Name,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "kind")]
pub enum Value {
    Variable(Variable),
    #[serde(rename = "IntValue")]
    Int {
        value: String,
    },
    #[serde(rename = "FloatValue")]
    Float {
        value: f64,
    },
    #[serde(rename = "ObjectValue")]
    Object {
        fields: Vec<ObjectField>,
    },
    #[serde(rename = "StringValue")]
    String {
        value: String,
        block: bool,
    },
    #[serde(rename = "ListValue")]
    List {
        values: Vec<Value>,
    },
    #[serde(rename = "BooleanValue")]
    Boolean {
        value: bool,
    },
    #[serde(rename = "NullValue")]
    Null,
    #[serde(rename = "EnumValue")]
    Enum {
        value: String,
    },
}

#[derive(Deserialize, Debug)]
pub struct Argument {
    name: Name,
    value: Value,
}

#[derive(Deserialize, Debug)]
pub struct NamedType {
    name: Name,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "kind")]
pub enum Type {
    #[serde(rename = "NamedType")]
    Named(NamedType),
    #[serde(rename = "NonNullType")]
    NonNull { value: Box<Type> },
    #[serde(rename = "ListType")]
    List { value: Box<Type> },
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct VariableDefinition {
    variable: Variable,
    #[serde(rename = "type")]
    of_type: Type,
    default_value: Option<Value>,
    directives: Vec<Directive>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "kind")]
pub enum Selection {
    #[serde(rename_all = "camelCase")]
    Field {
        alias: Option<Name>,
        name: Name,
        arguments: Vec<Argument>,
        directives: Vec<Directive>,
        selection_set: Option<SelectionSet>,
    },
    #[serde(rename_all = "camelCase")]
    InlineFragment {
        type_condition: Option<NamedType>,
        directives: Vec<Directive>,
        selection_set: SelectionSet,
    },
    #[serde(rename_all = "camelCase")]
    FragmentSpread {
        name: Name,
        directives: Vec<Directive>,
    },
}

#[derive(Deserialize, Debug)]
pub struct SelectionSet {
    selections: Vec<Selection>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "operation")]
#[serde(rename_all = "camelCase")]
pub enum Operation {
    SelectionSet(SelectionSet),
    #[serde(rename_all = "camelCase")]
    Query {
        name: Option<Name>,
        variable_definitions: Vec<VariableDefinition>,
        directives: Vec<Directive>,
        selection_set: SelectionSet,
    },
    #[serde(rename_all = "camelCase")]
    Mutation {
        name: Option<Name>,
        variable_definitions: Vec<VariableDefinition>,
        directives: Vec<Directive>,
        selection_set: SelectionSet,
    },
    #[serde(rename_all = "camelCase")]
    Subscription {
        name: Option<Name>,
        variable_definitions: Vec<VariableDefinition>,
        directives: Vec<Directive>,
        selection_set: SelectionSet,
    },
}

#[derive(Deserialize, Debug)]
#[serde(tag = "kind")]
pub enum Definition {
    #[serde(rename = "OperationDefinition")]
    Operation(Operation),
    #[serde(rename = "FragmentDefinition", rename_all = "camelCase")]
    Fragment {
        name: Name,
        type_condition: NamedType,
        directives: Vec<Directive>,
        selection_set: SelectionSet,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize() {
        let json_str = include_str!("../../fixtures/kitchen-sink_query.json");

        let doc: serde_json::Value = serde_json::from_str(json_str).expect("must be valid json");

        let doc: Vec<Definition> = serde_json::from_value(doc["definitions"].clone())
            .expect("kitchen sink query must be deserializable");

        insta::assert_debug_snapshot!(doc);
    }

    #[test]
    fn deserialize_name() {
        let json = serde_json::json!({
            "kind": "Name",
            "value": "queryName",
        });

        let result: Result<Name, _> = serde_json::from_value(json);

        match result {
            Err(err) => panic!("Deserialize failure {err}"),
            Ok(name) => assert_eq!(Name("queryName".to_owned()), name),
        };
    }

    #[test]
    fn deserialize_variable_definition() {
        let json = serde_json::json!({
          "kind": "VariableDefinition",
          "variable": {
            "kind": "Variable",
            "name": {
              "kind": "Name",
              "value": "input"
            }
          },
          "type": {
            "kind": "NamedType",
            "name": {
              "kind": "Name",
              "value": "StoryLikeSubscribeInput"
            }
          },
          "directives": []
        });

        let result: Result<VariableDefinition, _> = serde_json::from_value(json);

        if let Err(err) = result {
            panic!("Deserialize failure {err}");
        }
    }
}
