mod display;
mod from_graphql;

use std::fmt::Display;

use crate::app::config::TypescriptOptions;

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
pub struct UnionRef(String);

impl Ref for UnionRef {
    fn name(&self) -> String {
        format!("{}Union", self.0)
    }
}

#[derive(Clone)]
pub struct ObjectRef(String);

impl Ref for ObjectRef {
    fn name(&self) -> String {
        self.0.to_string()
    }
}

#[derive(Clone)]
pub enum TypeRef {
    Interface(InterfaceRef),
    Union(UnionRef),
    Object(ObjectRef),
}

impl Ref for TypeRef {
    fn name(&self) -> String {
        match self {
            Self::Interface(interface) => interface.name(),
            Self::Union(union) => union.name(),
            Self::Object(object) => object.name(),
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

pub struct Interface {
    name: String,
    doc_comment: Option<DocComment>,
    fields: Vec<Field>,
}

pub enum ScalarType {
    ID,
    String,
    Number,
    Boolean,
    Custom(Option<String>),
}

impl ScalarType {
    pub fn new(name: String, options: &TypescriptOptions) -> Self {
        match name.as_str() {
            "ID" => Self::ID,
            "String" => Self::String,
            "Float" | "Int" => Self::Number,
            "Boolean" => Self::Boolean,
            custom => Self::Custom(
                options
                    .scalar_newtypes
                    .as_ref()
                    .and_then(|nt| nt.get(custom).cloned()),
            ),
        }
    }
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
