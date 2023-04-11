use std::fmt::Display;

use crate::typescript::ts2::{self as ts, Ref};

impl Display for ts::Deprecable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Deprecated {
                message,
                description,
            } => {
                write!(f, "@deprecated")?;
                if let Some(message) = message {
                    write!(f, " {message}")?;
                }
                if let Some(description) = description {
                    write!(f, "\n\n{description}")?;
                }

                Ok(())
            }
            Self::Description(description) => {
                write!(f, "{description}")
            }
        }
    }
}

impl<T: Display> Display for ts::DocComment<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(comment) = self;
        let comment_string = comment.to_string();

        if comment_string.contains('\n') {
            write!(f, "/**\n * {}\n */", comment_string.replace('\n', "\n * "))
        } else {
            write!(f, "/* {comment_string} */")
        }
    }
}

impl Display for ts::Scalar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            name,
            doc_comment,
            of_type,
        } = self;
        if let Some(doc_comment) = doc_comment {
            writeln!(f, "{doc_comment}")?;
        }
        write!(f, "type {name}Scalar = ")?;
        let ts_type = match of_type {
            ts::ScalarType::ID => r#"NewType<string, "ID">"#.to_owned(),
            ts::ScalarType::String => "string".to_owned(),
            ts::ScalarType::Number => "number".to_owned(),
            ts::ScalarType::Boolean => "boolean".to_owned(),
            ts::ScalarType::Custom(Some(ts_type)) => ts_type.clone(),
            ts::ScalarType::Custom(None) => format!(r#"NewType<unknown, "{name}">"#),
        };
        writeln!(f, "{ts_type};")
    }
}

impl Display for ts::EnumValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ts::EnumValue { name, doc_comment } = self;

        if let Some(doc_comment) = doc_comment {
            writeln!(f, "{doc_comment}")?;
        }

        write!(f, "{name}")
    }
}

impl Display for ts::Enum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ts::Enum {
            name,
            doc_comment,
            values,
        } = self;

        if let Some(doc_comment) = doc_comment {
            writeln!(f, "{doc_comment}")?;
        }

        writeln!(
            f,
            "enum {name} = {{ {} }};",
            values
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

impl Display for ts::InterfaceRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl Display for ts::TypeRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl<R: ts::Ref + Display> Display for ts::NullableRefContainer<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ts::NullableRefContainer::Ref(r) => write!(f, "{r}"),
            ts::NullableRefContainer::List(r) => write!(f, "List<{r}>"),
        }
    }
}

impl<R: ts::Ref + Display> Display for ts::RefContainer<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ts::RefContainer::Ref(r) => write!(f, "{r}"),
            ts::RefContainer::List(r) => write!(f, "List<{r}>"),
            ts::RefContainer::Nullable(r) => write!(f, "Nullable<{r}>"),
        }
    }
}

impl Display for ts::Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            name,
            of_type,
            doc_comment,
        } = self;

        if let Some(doc_comment) = doc_comment {
            writeln!(f, "{doc_comment}")?;
        }

        writeln!(f, "{name}: {of_type}")
    }
}

impl Display for ts::Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            name,
            doc_comment,
            interfaces,
            fields,
        } = self;

        if let Some(doc_comment) = doc_comment {
            writeln!(f, "{doc_comment}")?;
        }

        let mut components = vec![format!(
            "{{ {} }}",
            fields
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        )];

        components.extend(interfaces.iter().map(ToString::to_string));

        writeln!(f, "type {name}Object = {};", components.join(" & "))
    }
}

impl Display for ts::Interface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            name,
            doc_comment,
            fields,
        } = self;

        if let Some(doc_comment) = doc_comment {
            writeln!(f, "{doc_comment}")?;
        }

        let fields = format!(
            "{{ {} }}",
            fields
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        );

        writeln!(f, "type {name}Interface = {};", fields)
    }
}

impl Display for ts::FieldedRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl Display for ts::Union {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            name,
            doc_comment,
            possible_types,
        } = self;

        if let Some(doc_comment) = doc_comment {
            writeln!(f, "{doc_comment}")?;
        }

        writeln!(
            f,
            "type {name}Union = {};",
            possible_types
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(" | ")
        )
    }
}

impl Display for ts::InputObjectRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}Input", self.0)
    }
}

impl Display for ts::ScalarRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}Scalar", self.0)
    }
}

impl Display for ts::InputRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ts::InputRef::InputObject(io) => io.fmt(f),
            ts::InputRef::Scalar(s) => s.fmt(f),
        }
    }
}

impl Display for ts::InputField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            name,
            doc_comment,
            of_type,
        } = self;
        if let Some(doc_comment) = doc_comment {
            writeln!(f, "{doc_comment}")?;
        }
        writeln!(f, "{name}: {of_type},")
    }
}

impl Display for ts::InputObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            name,
            doc_comment,
            input_fields,
        } = self;

        if let Some(doc_comment) = doc_comment {
            writeln!(f, "{doc_comment}")?;
        }

        write!(
            f,
            "type {name}Input = {{ {} }}",
            input_fields
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}
