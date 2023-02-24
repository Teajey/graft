use serde::{
    de::{self, Visitor},
    Deserialize,
};

#[derive(PartialEq, Debug)]
pub struct Name(String);

struct NameVisitor;

impl<'de> Visitor<'de> for NameVisitor {
    type Value = Name;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str(r#"a map { kind: "Name", value: <some name> }"#)
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let mut kind = None;
        let mut value = None;

        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "kind" => {
                    if kind.is_some() {
                        return Err(de::Error::duplicate_field("kind"));
                    }
                    kind = Some(map.next_value::<String>()?);
                }
                "value" => {
                    if value.is_some() {
                        return Err(de::Error::duplicate_field("value"));
                    }
                    value = Some(map.next_value()?);
                }
                x => return Err(de::Error::unknown_field(x, &["kind", "value"])),
            }
        }

        let Some("Name") = kind.as_deref() else {
            return Err(de::Error::custom(r#""kind" must be string "Name""#));
        };

        let Some(value) = value else {
            return Err(de::Error::missing_field("value"))
        };

        Ok(Name(value))
    }
}

impl<'de> Deserialize<'de> for Name {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(NameVisitor)
    }
}

#[derive(Deserialize, Debug)]
pub struct Directive {
    name: Name,
    arguments: Vec<Kind>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "kind")]
pub enum Kind {
    Argument {
        name: Name,
        value: Box<Kind>,
    },
    ListValue {
        values: Vec<Kind>,
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
        variable: Box<Kind>,
        #[serde(rename = "type")]
        of_type: Box<Kind>,
        default_value: Option<Box<Kind>>,
        directives: Vec<Directive>,
    },
    ObjectField {
        name: Name,
        value: Box<Kind>,
    },
    ObjectValue {
        fields: Vec<Kind>,
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
        arguments: Vec<Kind>,
        directives: Vec<Directive>,
        selection_set: Option<Box<SelectionSet>>,
    },
    #[serde(rename_all = "camelCase")]
    InlineFragment {
        type_condition: Kind,
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
        variable_definitions: Vec<Kind>,
        directives: Vec<Directive>,
        selection_set: SelectionSet,
    },
    #[serde(rename_all = "camelCase")]
    Subscription {
        name: Name,
        variable_definitions: Vec<Kind>,
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
        type_condition: Kind,
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

        let result: Result<Kind, _> = serde_json::from_value(json);

        if let Err(err) = result {
            panic!("Deserialize failure {err}");
        }
    }
}
