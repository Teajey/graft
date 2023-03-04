mod from_graphql;

use eyre::{eyre, Result};

use crate::{
    app::config::TypescriptOptions,
    graphql::{
        query::{self, OperationType},
        schema::Schema,
    },
    util::Named,
};

#[derive(Clone)]
pub struct Argument<'t> {
    name: String,
    description: Option<String>,
    of_type: &'t InputType<'t>,
}

#[derive(Clone)]
pub struct Arguments<'t>(Vec<Argument<'t>>);

#[derive(Clone)]
pub struct FieldName(String);

#[derive(Clone)]
pub struct Field<'t> {
    name: FieldName,
    description: Option<String>,
    of_type: Type<'t>,
}

#[derive(Clone)]
pub struct Interface<'t> {
    name: String,
    description: Option<String>,
    fields: Vec<Field<'t>>,
    possible_types: Vec<&'t Type<'t>>,
}

#[derive(Clone)]
pub struct Object<'t> {
    name: String,
    description: Option<String>,
    interfaces: Vec<&'t Interface<'t>>,
    fields: Vec<Field<'t>>,
}

#[derive(Clone)]
pub struct Scalar {
    name: String,
    description: Option<String>,
}

pub struct InputObject<'t> {
    name: String,
    description: Option<String>,
    input_fields: Arguments<'t>,
}

#[derive(Clone)]
pub struct Union<'t> {
    name: String,
    description: Option<String>,
    possible_types: Vec<&'t Object<'t>>,
}

#[derive(Clone)]
pub struct EnumValue {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Clone)]
pub struct Enum {
    pub name: String,
    pub description: Option<String>,
    pub enum_values: Vec<EnumValue>,
}

pub enum InputType<'t> {
    InputObject(InputObject<'t>),
    Scalar(Scalar),
}

#[derive(Clone)]
pub enum Type<'t> {
    Named(&'t NamedType<'t>),
    List(Box<Type<'t>>),
    NonNull(Box<Type<'t>>),
}

#[derive(Clone)]
pub struct SelectionName<'t> {
    name: &'t FieldName,
    alias: Option<String>,
}

#[derive(Clone)]
pub enum FieldedType<'t> {
    Union(Union<'t>),
    Object(Object<'t>),
    Interface(Interface<'t>),
}

#[derive(Clone)]
pub enum LeafType {
    Scalar(Scalar),
    Enum(Enum),
}

#[derive(Clone)]
pub enum NamedType<'t> {
    Fielded(FieldedType<'t>),
    Leaf(LeafType),
}

impl<'t> FieldedType<'t> {
    pub fn get_field(&self, name: &str) -> Option<&Field> {
        match self {
            FieldedType::Union(uni) => uni
                .possible_types
                .iter()
                .flat_map(|t| &t.fields)
                .find(|field| field.name.0 == name),
            FieldedType::Object(Object { fields, .. })
            | FieldedType::Interface(Interface { fields, .. }) => {
                fields.iter().find(|field| field.name.0 == name)
            }
        }
    }
}

#[derive(Clone)]
pub enum NonNullSelectionType<'t> {
    SelectionSet(SelectionSet<'t>),
    List(Box<ListSelectionType<'t>>),
}

#[derive(Clone)]
pub enum ListSelectionType<'t> {
    SelectionSet(SelectionSet<'t>),
    NonNull(NonNullSelectionType<'t>),
    List(Box<ListSelectionType<'t>>),
}

#[derive(Clone)]
pub enum NamedSelectionType<'t> {
    SelectionSet(SelectionSet<'t>),
    On(&'t FieldedType<'t>),
}

#[derive(Clone)]
pub enum SelectionType<'t> {
    Named(NamedSelectionType<'t>),
    List(ListSelectionType<'t>),
    NonNull(NonNullSelectionType<'t>),
}

#[derive(Clone)]
pub struct Selection<'t> {
    name: SelectionName<'t>,
    of_type: SelectionType<'t>,
}

#[derive(Clone)]
pub struct SelectionSet<'t>(Vec<Selection<'t>>);

pub struct Fragment<'t> {
    name: String,
    type_condition: &'t FieldedType<'t>,
    selection_set: SelectionSet<'t>,
    doc: query::Fragment,
}

pub struct Operation<'t> {
    of_type: OperationType,
    name: String,
    arguments: Arguments<'t>,
    selection_set: SelectionSet<'t>,
    doc: query::Operation,
}

// struct GraphQLTypescript<'t> {
//     types: TypesIndex<'t>,
//     operations: Vec<Operation<'t>>,
//     fragments: FragmentsIndex<'t>,
// }

// impl<'t> GraphQLTypescript<'t> {
//     pub fn try_new(
//         options: TypescriptOptions,
//         schema: Schema,
//         definitions: Vec<query::Definition>,
//     ) -> Result<Self> {
//         let mut types = TypesIndex(HashMap::new());

//         for t in schema.types {
//             types.0.insert(t.name().to_owned(), t.into());
//         }

//         let mut operations = Vec::new();
//         let mut fragments = HashMap::new();

//         let types_ref = &types;

//         for definition in definitions {
//             match definition {
//                 query::Definition::Operation(operation) => {
//                     operations.push(operation);
//                 }
//                 query::Definition::Fragment(fragment) => {
//                     fragments.insert(
//                         fragment.name.0.clone(),
//                         types_ref.with(fragment).try_into()?,
//                     );
//                 }
//             }
//         }

//         let ctx = Context {
//             options,
//             types,
//             fragments,
//         };

//         let operations = operations
//             .into_iter()
//             .map(|op| ctx.with(op).try_into())
//             .collect::<Result<_>>()?;

//         Ok(Self {
//             types: ctx.types,
//             operations,
//             fragments: ctx.fragments,
//         })
//     }
// }
