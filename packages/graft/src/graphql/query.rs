mod from_ast;

use std::fmt::Display;

use serde::{ser::SerializeMap, Serialize};

#[derive(PartialEq, Debug, Clone)]
pub struct Name(pub String);

impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

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

#[derive(Debug, Serialize, Clone)]
#[cfg_attr(test, derive(serde::Deserialize))]
pub struct Directive {
    kind: Option<tag::Directive>,
    name: Name,
    arguments: Vec<Argument>,
}

#[derive(Debug, Serialize, Clone)]
#[cfg_attr(test, derive(serde::Deserialize))]
pub struct ObjectField {
    kind: Option<tag::ObjectField>,
    name: Name,
    value: Value,
}

#[derive(Debug, Serialize, Clone)]
#[cfg_attr(test, derive(serde::Deserialize))]
pub struct Variable {
    kind: tag::Variable,
    pub name: Name,
}

#[derive(Debug, Serialize, Clone)]
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
    #[derive(Debug, serde::Serialize, Clone)]
    #[cfg_attr(test, derive(serde::Deserialize))]
    pub enum Argument {
        #[serde(rename = "Argument")]
        T,
    }

    #[derive(Debug, serde::Serialize, Clone)]
    #[cfg_attr(test, derive(serde::Deserialize))]
    pub enum VariableDefinition {
        #[serde(rename = "VariableDefinition")]
        T,
    }

    #[derive(Debug, serde::Serialize, Clone)]
    #[cfg_attr(test, derive(serde::Deserialize))]
    pub enum Variable {
        #[serde(rename = "Variable")]
        T,
    }

    #[derive(Debug, serde::Serialize, Clone)]
    #[cfg_attr(test, derive(serde::Deserialize))]
    pub enum Directive {
        #[serde(rename = "Directive")]
        T,
    }

    #[derive(Debug, serde::Serialize, Clone)]
    #[cfg_attr(test, derive(serde::Deserialize))]
    pub enum ObjectField {
        #[serde(rename = "ObjectField")]
        T,
    }

    #[derive(Debug, serde::Serialize, Clone)]
    #[cfg_attr(test, derive(serde::Deserialize))]
    pub enum SelectionSet {
        #[serde(rename = "SelectionSet")]
        T,
    }

    #[derive(Debug, serde::Serialize, Clone)]
    #[cfg_attr(test, derive(serde::Deserialize))]
    pub enum NamedType {
        #[serde(rename = "NamedType")]
        T,
    }

    #[derive(Debug, serde::Serialize, Clone)]
    #[cfg_attr(test, derive(serde::Deserialize))]
    pub enum Document {
        #[serde(rename = "Document")]
        T,
    }
}

#[derive(Debug, Serialize, Clone)]
#[cfg_attr(test, derive(serde::Deserialize))]
pub struct Argument {
    kind: Option<tag::Argument>,
    name: Name,
    value: Value,
}

#[derive(Debug, Serialize, Clone)]
#[cfg_attr(test, derive(serde::Deserialize))]
pub struct NamedType {
    kind: Option<tag::NamedType>,
    pub name: Name,
}

#[derive(Debug, Serialize, Clone)]
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

#[derive(Debug, Serialize, Clone)]
#[cfg_attr(test, derive(serde::Deserialize))]
#[serde(rename_all = "camelCase")]
pub struct VariableDefinition {
    kind: Option<tag::VariableDefinition>,
    pub variable: Variable,
    #[serde(rename = "type")]
    pub of_type: Type,
    pub default_value: Option<Value>,
    pub directives: Vec<Directive>,
}

#[derive(Debug, Serialize, Clone)]
#[cfg_attr(test, derive(serde::Deserialize))]
pub struct VariableDefinitions(pub Vec<VariableDefinition>);

#[derive(Debug, Serialize, Clone)]
#[cfg_attr(test, derive(serde::Deserialize))]
#[serde(rename_all = "camelCase")]
pub struct Field {
    pub alias: Option<Name>,
    pub name: Name,
    pub arguments: Vec<Argument>,
    pub directives: Vec<Directive>,
    pub selection_set: Option<SelectionSet>,
}

#[derive(Debug, Serialize, Clone)]
#[cfg_attr(test, derive(serde::Deserialize))]
#[serde(rename_all = "camelCase")]
pub struct InlineFragment {
    type_condition: Option<NamedType>,
    directives: Vec<Directive>,
    selection_set: SelectionSet,
}

#[derive(Debug, Serialize, Clone)]
#[cfg_attr(test, derive(serde::Deserialize))]
#[serde(rename_all = "camelCase")]
pub struct FragmentSpread {
    name: Name,
    directives: Vec<Directive>,
}

#[derive(Debug, Serialize, Clone)]
#[cfg_attr(test, derive(serde::Deserialize))]
#[serde(tag = "kind")]
pub enum Selection {
    Field(Field),
    InlineFragment(InlineFragment),
    FragmentSpread(FragmentSpread),
}

#[derive(Debug, Serialize, Clone)]
#[cfg_attr(test, derive(serde::Deserialize))]
pub struct SelectionSet {
    kind: Option<tag::SelectionSet>,
    pub selections: Vec<Selection>,
}

#[derive(Debug, Serialize, Clone)]
#[cfg_attr(test, derive(serde::Deserialize))]
#[serde(rename_all = "camelCase")]
pub enum OperationType {
    Query,
    Mutation,
    Subscription,
}

#[derive(Debug, Serialize, Clone)]
#[cfg_attr(test, derive(serde::Deserialize))]
#[serde(rename_all = "camelCase")]
pub struct NamedOperation {
    pub operation: OperationType,
    pub name: Option<Name>,
    pub variable_definitions: VariableDefinitions,
    pub directives: Vec<Directive>,
    pub selection_set: SelectionSet,
}

#[derive(Debug, Serialize, Clone)]
#[cfg_attr(test, derive(serde::Deserialize))]
#[serde(untagged)]
pub enum Operation {
    SelectionSet(SelectionSet),
    NamedOperation(NamedOperation),
}

#[derive(Debug, Serialize, Clone)]
#[cfg_attr(test, derive(serde::Deserialize))]
#[serde(rename_all = "camelCase")]
pub struct Fragment {
    pub name: Name,
    pub type_condition: NamedType,
    pub directives: Vec<Directive>,
    pub selection_set: SelectionSet,
}

#[derive(Debug, Serialize, Clone)]
#[cfg_attr(test, derive(serde::Deserialize))]
#[serde(tag = "kind")]
pub enum Definition {
    #[serde(rename = "OperationDefinition")]
    Operation(Operation),
    #[serde(rename = "FragmentDefinition")]
    Fragment(Fragment),
}

#[derive(Debug, Serialize, Clone)]
#[cfg_attr(test, derive(serde::Deserialize))]
pub struct Document {
    kind: tag::Document,
    pub definitions: Vec<Definition>,
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
        );
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
