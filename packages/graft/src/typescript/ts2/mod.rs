mod display;
mod from_graphql;

use std::fmt::Display;

use eyre::{eyre, Result};

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

pub trait Ref: Sized {
    fn name(&self) -> String;

    fn try_from_type_ref(type_ref: TypeRef) -> Result<Self>;
}

#[derive(Clone, Debug)]
pub struct InterfaceRef(String);

impl Ref for InterfaceRef {
    fn name(&self) -> String {
        format!("{}Interface", self.0)
    }

    fn try_from_type_ref(type_ref: TypeRef) -> Result<Self> {
        match type_ref {
            TypeRef::Interface(i) => Ok(i),
            unexpected => Err(eyre!("Expected InterfaceRef but found {unexpected:?}")),
        }
    }
}

#[derive(Clone, Debug)]
pub struct UnionRef(String);

impl Ref for UnionRef {
    fn name(&self) -> String {
        format!("{}Union", self.0)
    }

    fn try_from_type_ref(type_ref: TypeRef) -> Result<Self> {
        match type_ref {
            TypeRef::Union(u) => Ok(u),
            unexpected => Err(eyre!("Expected UnionRef but found {unexpected:?}")),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ObjectRef(String);

impl Ref for ObjectRef {
    fn name(&self) -> String {
        self.0.to_string()
    }

    fn try_from_type_ref(type_ref: TypeRef) -> Result<Self> {
        match type_ref {
            TypeRef::Object(o) => Ok(o),
            unexpected => Err(eyre!("Expected ObjectRef but found {unexpected:?}")),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ScalarRef(String);

impl Ref for ScalarRef {
    fn name(&self) -> String {
        format!("{}Scalar", self.0)
    }

    fn try_from_type_ref(type_ref: TypeRef) -> Result<Self> {
        match type_ref {
            TypeRef::Scalar(s) => Ok(s),
            unexpected => Err(eyre!("Expected ScalarRef but found {unexpected:?}")),
        }
    }
}

#[derive(Clone, Debug)]
pub enum TypeRef {
    Interface(InterfaceRef),
    Union(UnionRef),
    Object(ObjectRef),
    Scalar(ScalarRef),
    InputObject(InputObjectRef),
}

impl TypeRef {
    fn try_into_ref<R: Ref>(self) -> Result<R> {
        R::try_from_type_ref(self)
    }
}

impl Ref for TypeRef {
    fn name(&self) -> String {
        match self {
            Self::Interface(interface) => interface.name(),
            Self::Union(union) => union.name(),
            Self::Object(object) => object.name(),
            Self::Scalar(scalar) => scalar.name(),
            Self::InputObject(io) => io.name(),
        }
    }

    fn try_from_type_ref(type_ref: TypeRef) -> Result<Self> {
        Ok(type_ref)
    }
}

#[derive(Clone)]
pub enum FieldedRef {
    Interface(InterfaceRef),
    Union(UnionRef),
    Object(ObjectRef),
}

impl Ref for FieldedRef {
    fn name(&self) -> String {
        match self {
            Self::Interface(interface) => interface.name(),
            Self::Union(union) => union.name(),
            Self::Object(object) => object.name(),
        }
    }

    fn try_from_type_ref(type_ref: TypeRef) -> Result<Self> {
        let fielded_ref = match type_ref {
            TypeRef::Interface(i) => Self::Interface(i),
            TypeRef::Union(u) => Self::Union(u),
            TypeRef::Object(o) => Self::Object(o),
            unexpected => return Err(eyre!("Expected FieldedRef but found {unexpected:?}")),
        };

        Ok(fielded_ref)
    }
}

#[derive(Clone, Debug)]
pub struct InputObjectRef(String);

impl Ref for InputObjectRef {
    fn name(&self) -> String {
        format!("{}Input", self.0)
    }

    fn try_from_type_ref(type_ref: TypeRef) -> Result<Self> {
        match type_ref {
            TypeRef::InputObject(io) => Ok(io),
            unexpected => Err(eyre!("Expected InputObjectRef but found {unexpected:?}")),
        }
    }
}

#[derive(Clone)]
pub enum InputRef {
    InputObject(InputObjectRef),
    Scalar(ScalarRef),
}

impl Ref for InputRef {
    fn name(&self) -> String {
        match self {
            Self::InputObject(io) => io.name(),
            Self::Scalar(s) => s.name(),
        }
    }

    fn try_from_type_ref(type_ref: TypeRef) -> Result<Self> {
        let input_ref = match type_ref {
            TypeRef::Scalar(s) => Self::Scalar(s),
            TypeRef::InputObject(io) => Self::InputObject(io),
            unexpected => return Err(eyre!("Expected InputRef but found {unexpected:?}")),
        };

        Ok(input_ref)
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

pub struct Union {
    name: String,
    doc_comment: Option<DocComment>,
    possible_types: Vec<FieldedRef>,
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

pub struct InputField {
    name: String,
    doc_comment: Option<DocComment>,
    of_type: RefContainer<InputRef>,
}

pub struct InputObject {
    name: String,
    doc_comment: Option<DocComment>,
    input_fields: Vec<InputField>,
}
