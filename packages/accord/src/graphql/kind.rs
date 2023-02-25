pub use macros::Kind;
use serde::{de, Deserialize};

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
