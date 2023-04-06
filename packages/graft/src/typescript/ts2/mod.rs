mod display;
mod from_graphql;

use std::fmt::Display;

#[derive(Clone)]
pub struct DocComment<T: Display = String>(T);

impl DocComment<Deprecable> {
    pub fn maybe_new(
        is_deprecated: bool,
        deprecation_reason: Option<String>,
        description: Option<String>,
    ) -> Option<Self> {
        let deprecable = if is_deprecated {
            Deprecable::Deprecated {
                message: deprecation_reason,
                description,
            }
        } else {
            Deprecable::Description(description?)
        };

        Some(Self(deprecable))
    }
}

#[derive(Clone)]
pub enum Deprecable {
    Deprecated {
        message: Option<String>,
        description: Option<String>,
    },
    Description(String),
}

pub trait Ref {
    fn name(&self) -> String;
}

#[derive(Clone)]
pub struct InterfaceRef(String);

impl Ref for InterfaceRef {
    fn name(&self) -> String {
        format!("{}Interface", self.0)
    }
}

#[derive(Clone)]
pub enum TypeRef {
    Interface(InterfaceRef),
}

impl Ref for TypeRef {
    fn name(&self) -> String {
        match self {
            Self::Interface(interface) => interface.name(),
        }
    }
}

#[derive(Clone)]
pub enum NullableRefContainer<R: Ref> {
    Ref(R),
    List(Box<RefContainer<R>>),
}

#[derive(Clone)]
pub enum RefContainer<R: Ref> {
    Ref(R),
    List(Box<RefContainer<R>>),
    Nullable(NullableRefContainer<R>),
}

#[derive(Clone)]
pub struct Field {
    pub name: String,
    pub doc_comment: Option<DocComment<Deprecable>>,
    pub of_type: RefContainer<TypeRef>,
}

#[derive(Clone)]
pub struct Object {
    name: String,
    doc_comment: Option<DocComment>,
    interfaces: Vec<InterfaceRef>,
    fields: Vec<Field>,
}

pub enum ScalarType {
    ID,
    String,
    Number,
    Boolean,
    Custom(Option<String>),
}

pub struct Scalar {
    name: String,
    doc_comment: Option<DocComment>,
    of_type: ScalarType,
}

pub struct EnumValue {
    name: String,
    doc_comment: Option<DocComment<Deprecable>>,
}

pub struct Enum {
    name: String,
    doc_comment: Option<DocComment>,
    values: Vec<EnumValue>,
}
