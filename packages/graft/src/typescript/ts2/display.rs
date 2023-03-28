use std::{fmt::Display, marker::PhantomData};

use crate::{
    app::config::TypescriptOptions,
    graphql::schema,
    typescript::ts::{self, Comment, Typescript},
};

trait TypeRefSuffix {
    const SUFFIX: &'static str;
}

struct Interface;

impl TypeRefSuffix for Interface {
    const SUFFIX: &'static str = "Interface";
}

impl Display for Comment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(comment_string) = self;

        if comment_string.contains('\n') {
            write!(f, "/**\n * {}\n */", comment_string.replace('\n', "\n * "))
        } else {
            write!(f, "/* {comment_string} */")
        }
    }
}

impl<'a> Display for Typescript<(schema::named_type::Scalar, &'a TypescriptOptions)> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Typescript((schema::named_type::Scalar { name, description }, options)) = self;
        if let Some(description) = description {
            writeln!(f, "{}", Comment(description.clone()))?;
        }
        let scalar_type = match name.as_str() {
            "ID" => r#"NewType<string, "ID">"#.to_owned(),
            "String" => "string".to_owned(),
            "Int" | "Float" => "number".to_owned(),
            "Boolean" => "boolean".to_owned(),
            name => {
                let default = || format!(r#"NewType<unknown, "{name}">"#);
                match &options.scalar_newtypes {
                    None => default(),
                    Some(scalar_newtypes) => {
                        scalar_newtypes.get(name).cloned().unwrap_or_else(default)
                    }
                }
            }
        };
        writeln!(f, "type {name}Scalar = {scalar_type};",)
    }
}

impl<'a> Display for Typescript<&'a schema::EnumValue> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Typescript(schema::EnumValue {
            name,
            description,
            is_deprecated,
            deprecation_reason,
        }) = self;

        if let Some(description) = description {
            writeln!(
                f,
                "{}",
                Comment::maybe_deprecated(
                    *is_deprecated,
                    deprecation_reason.as_deref(),
                    description.clone()
                )
            )?;
        }

        write!(f, "{name}")
    }
}

impl Display for Typescript<schema::named_type::Enum> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Typescript(schema::named_type::Enum {
            name,
            description,
            enum_values,
        }) = self;

        if let Some(description) = description {
            writeln!(f, "{}", Comment(description.clone()))?;
        }

        writeln!(
            f,
            "enum {name} = {{ {} }};",
            enum_values
                .iter()
                .map(|ev| Typescript(ev).to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

impl<'a> Display for Typescript<&'a schema::Field> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl<'a, S: TypeRefSuffix> Display for Typescript<(ts::NullableTypeRef, PhantomData<S>)> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Typescript((type_ref, suffix)) = self;
        match type_ref {
            ts::NullableTypeRef::To { name } => write!(f, "{name}{suffix}", suffix = S::SUFFIX),
            ts::NullableTypeRef::List(type_ref) => {
                write!(
                    f,
                    "List<{type_ref}>",
                    type_ref = Typescript((**type_ref, *suffix))
                )
            }
        }
    }
}

impl<'a, S: TypeRefSuffix> Display for Typescript<(ts::TypeRef, PhantomData<S>)> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Typescript((type_ref, suffix)) = self;
        match type_ref {
            ts::TypeRef::To { name } => write!(f, "{name}{}", S::SUFFIX),
            ts::TypeRef::List(type_ref) => write!(
                f,
                "List<{type_ref}>",
                type_ref = Typescript((**type_ref, *suffix))
            ),
            ts::TypeRef::Nullable(type_ref) => {
                write!(
                    f,
                    "Nullable<{type_ref}>",
                    type_ref = Typescript((*type_ref, *suffix))
                )
            }
        }
    }
}

impl Display for Typescript<schema::named_type::Object> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Typescript(schema::named_type::Object {
            name,
            description,
            fields,
            interfaces,
        }) = self;

        if let Some(description) = description {
            writeln!(f, "{}", Comment(description.clone()))?;
        }

        let mut components = vec![format!(
            "{{ {} }}",
            fields
                .iter()
                .map(|f| Typescript(f).to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )];

        components.extend(interfaces.iter().map(|i| {
            Typescript((ts::TypeRef::from(i.clone()), PhantomData::<Interface>)).to_string()
        }));

        writeln!(f, "type {name}Object = {};", components.join(" & "))
    }
}
