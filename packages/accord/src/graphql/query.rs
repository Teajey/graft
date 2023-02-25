mod from_ast;

use serde::{ser::SerializeMap, Serialize};

#[derive(PartialEq, Debug)]
pub struct Name(pub String);

impl Serialize for Name {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("kind", "Name")?;
        map.serialize_entry("value", &self.0)?;
        map.end()
    }
}

#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(serde::Deserialize))]
pub struct Directive {
    kind: Option<tag::Directive>,
    name: Name,
    arguments: Vec<Argument>,
}

#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(serde::Deserialize))]
pub struct ObjectField {
    kind: Option<tag::ObjectField>,
    name: Name,
    value: Value,
}

#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(serde::Deserialize))]
pub struct Variable {
    kind: tag::Variable,
    name: Name,
}

#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(serde::Deserialize))]
#[serde(tag = "kind")]
pub enum Value {
    Variable {
        name: Name,
    },
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

// FIXME: I shouldn't need this :'(
pub mod tag {
    #[derive(Debug, serde::Serialize)]
    #[cfg_attr(test, derive(serde::Deserialize))]
    pub enum Argument {
        #[serde(rename = "Argument")]
        T,
    }

    #[derive(Debug, serde::Serialize)]
    #[cfg_attr(test, derive(serde::Deserialize))]
    pub enum VariableDefinition {
        #[serde(rename = "VariableDefinition")]
        T,
    }

    #[derive(Debug, serde::Serialize)]
    #[cfg_attr(test, derive(serde::Deserialize))]
    pub enum Variable {
        #[serde(rename = "Variable")]
        T,
    }

    #[derive(Debug, serde::Serialize)]
    #[cfg_attr(test, derive(serde::Deserialize))]
    pub enum Directive {
        #[serde(rename = "Directive")]
        T,
    }

    #[derive(Debug, serde::Serialize)]
    #[cfg_attr(test, derive(serde::Deserialize))]
    pub enum ObjectField {
        #[serde(rename = "ObjectField")]
        T,
    }

    #[derive(Debug, serde::Serialize)]
    #[cfg_attr(test, derive(serde::Deserialize))]
    pub enum SelectionSet {
        #[serde(rename = "SelectionSet")]
        T,
    }

    #[derive(Debug, serde::Serialize)]
    #[cfg_attr(test, derive(serde::Deserialize))]
    pub enum NamedType {
        #[serde(rename = "NamedType")]
        T,
    }

    #[derive(Debug, serde::Serialize)]
    #[cfg_attr(test, derive(serde::Deserialize))]
    pub enum Document {
        #[serde(rename = "Document")]
        T,
    }
}

#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(serde::Deserialize))]
pub struct Argument {
    kind: Option<tag::Argument>,
    name: Name,
    value: Value,
}

#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(serde::Deserialize))]
pub struct NamedType {
    kind: Option<tag::NamedType>,
    name: Name,
}

#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(serde::Deserialize))]
#[serde(tag = "kind")]
pub enum Type {
    #[serde(rename = "NamedType")]
    Named { name: Name },
    #[serde(rename = "NonNullType")]
    NonNull { value: Box<Type> },
    #[serde(rename = "ListType")]
    List { value: Box<Type> },
}

#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(serde::Deserialize))]
#[serde(rename_all = "camelCase")]
pub struct VariableDefinition {
    kind: Option<tag::VariableDefinition>,
    variable: Variable,
    #[serde(rename = "type")]
    of_type: Type,
    default_value: Option<Value>,
    directives: Vec<Directive>,
}

#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(serde::Deserialize))]
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

#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(serde::Deserialize))]
pub struct SelectionSet {
    kind: Option<tag::SelectionSet>,
    selections: Vec<Selection>,
}

#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(serde::Deserialize))]
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

#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(serde::Deserialize))]
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

#[derive(Debug, Serialize)]
#[cfg_attr(test, derive(serde::Deserialize))]
pub struct Document {
    kind: tag::Document,
    definitions: Vec<Definition>,
}

impl Document {
    pub fn new(definitions: Vec<Definition>) -> Self {
        Self {
            kind: tag::Document::T,
            definitions,
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::{assert_eq, assert_str_eq};

    use super::*;

    #[test]
    fn deserialize() {
        let json_str = include_str!("../../fixtures/kitchen-sink_query.json");

        let doc: serde_json::Value = serde_json::from_str(json_str).expect("must be valid json");

        let defs: Vec<Definition> = serde_json::from_value(doc["definitions"].clone())
            .expect("kitchen sink query must be deserializable");

        insta::assert_debug_snapshot!(defs);
    }

    #[test]
    fn lossless() {
        let json_str = include_str!("../../fixtures/kitchen-sink_query.json");

        let doc: serde_json::Value = serde_json::from_str(json_str).expect("must be valid json");

        let defs: Vec<Definition> = serde_json::from_value(doc["definitions"].clone())
            .expect("kitchen sink query must be deserializable");

        let re_json = serde_json::json!({
            "kind": "Document",
            "definitions": defs,
        });

        insta::assert_snapshot!(
            serde_json::to_string_pretty(&re_json).expect("failed to pretty print re_json")
        )
    }

    #[test]
    fn deserialize_name() {
        let json = serde_json::json!({
            "kind": "Name",
            "value": "queryName",
        });

        let name: Name = serde_json::from_value(json).expect("Failed to deserialize");

        assert_eq!(Name("queryName".to_owned()), name);
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
          "directives": [],
          "defaultValue": null,
        });

        let json_str = serde_json::to_string_pretty(&json).expect("Failed to pretty print json");

        let vd: VariableDefinition = serde_json::from_value(json).expect("Failed to deserialize");

        assert_str_eq!(
            json_str,
            serde_json::to_string_pretty(&serde_json::json!(vd))
                .expect("Failed to pretty print vd")
        );
    }
}
