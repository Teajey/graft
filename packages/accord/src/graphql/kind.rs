pub use macros::Kind;
use serde::{de, Deserialize};

use super::query::Name;

pub trait Kind: for<'de> Deserialize<'de> {}

pub fn visit_map<'de, A>(invalid_kind_msg: &'static str, mut map: A) -> Result<String, A::Error>
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
        return Err(de::Error::custom(invalid_kind_msg));
    };

    let Some(value) = value else {
        return Err(de::Error::missing_field("value"))
    };

    Ok(value)
}

impl crate::graphql::kind::Kind for super::query::Name {}
struct NameVisitor;
impl<'de> serde::de::Visitor<'de> for NameVisitor {
    type Value = Name;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "a map {{ kind: \"{}\", value: <some name> }}",
            stringify!(Name)
        )
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        let value = crate::graphql::kind::visit_map("\"kind\" must be string \"#name\"", map)?;
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
