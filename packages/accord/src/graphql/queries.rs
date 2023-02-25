use serde::Deserialize;

use super::kind::Kind;

#[derive(PartialEq, Debug, Kind)]
pub struct Name(String);

#[derive(Deserialize, Debug)]
pub struct Directive {
    name: Name,
    arguments: Vec<KindEnum>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "kind")]
pub enum KindEnum {
    Argument {
        name: Name,
        value: Box<KindEnum>,
    },
    ListValue {
        values: Vec<KindEnum>,
    },
    IntValue {
        value: String,
    },
    NamedType {
        name: Name,
    },
    BooleanValue {
        value: bool,
    },
    NullValue,
    Variable {
        name: Name,
    },
    EnumValue {
        value: String,
    },
    #[serde(rename_all = "camelCase")]
    VariableDefinition {
        variable: Box<KindEnum>,
        #[serde(rename = "type")]
        of_type: Box<KindEnum>,
        default_value: Option<Box<KindEnum>>,
        directives: Vec<Directive>,
    },
    ObjectField {
        name: Name,
        value: Box<KindEnum>,
    },
    ObjectValue {
        fields: Vec<KindEnum>,
    },
    StringValue {
        value: String,
        block: bool,
    },
}

#[derive(Deserialize, Debug)]
#[serde(tag = "kind")]
pub enum Selection {
    #[serde(rename_all = "camelCase")]
    Field {
        alias: Option<Name>,
        name: Name,
        arguments: Vec<KindEnum>,
        directives: Vec<Directive>,
        selection_set: Option<Box<SelectionSet>>,
    },
    #[serde(rename_all = "camelCase")]
    InlineFragment {
        type_condition: KindEnum,
        directives: Vec<Directive>,
        selection_set: Box<SelectionSet>,
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
    Query {
        name: Option<Name>,
        directives: Vec<Directive>,
    },
    #[serde(rename_all = "camelCase")]
    Mutation {
        name: Name,
        variable_definitions: Vec<KindEnum>,
        directives: Vec<Directive>,
        selection_set: SelectionSet,
    },
    #[serde(rename_all = "camelCase")]
    Subscription {
        name: Name,
        variable_definitions: Vec<KindEnum>,
        directives: Vec<Directive>,
        selection_set: SelectionSet,
    },
}

#[derive(Deserialize, Debug)]
#[serde(tag = "kind")]
pub enum Definition {
    OperationDefinition(Operation),
    #[serde(rename_all = "camelCase")]
    FragmentDefinition {
        name: Name,
        type_condition: KindEnum,
        directives: Vec<Directive>,
        selection_set: SelectionSet,
    },
}

#[derive(Deserialize, Debug)]
#[serde(tag = "kind")]
enum Document {
    Document { definitions: Vec<Definition> },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize() {
        let json_str = include_str!("../../fixtures/kitchen-sink_query.json");

        let definition: serde_json::Value =
            serde_json::from_str(json_str).expect("must be valid json");

        let result: Result<Document, _> = serde_json::from_value(definition);

        match result {
            Err(err) => panic!("Deserialize failure {err}"),
            Ok(doc) => insta::assert_debug_snapshot!(doc),
        }
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

        let result: Result<KindEnum, _> = serde_json::from_value(json);

        if let Err(err) = result {
            panic!("Deserialize failure {err}");
        }
    }
}
