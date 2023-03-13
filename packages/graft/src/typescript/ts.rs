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
    of_type: Type<'t, InputType<'t>>,
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
    possible_types: Vec<&'t Object<'t>>,
}

impl<'t> PossibleTypes for Interface<'t> {
    fn get_possible_types(&self) -> &[&Object] {
        &self.possible_types
    }
}

impl<'t> Fielded<'t> for Interface<'t> {
    fn get_fields(&'t self) -> Vec<&'t Field<'t>> {
        self.fields.iter().collect::<Vec<_>>()
    }
}

#[derive(Clone)]
pub struct Object<'t> {
    name: String,
    description: Option<String>,
    interfaces: Vec<&'t Interface<'t>>,
    fields: Vec<Field<'t>>,
}

impl<'t> Fielded<'t> for Object<'t> {
    fn get_fields(&'t self) -> Vec<&'t Field<'t>> {
        self.fields.iter().collect::<Vec<_>>()
    }
}

#[derive(Clone)]
pub struct Scalar {
    name: String,
    description: Option<String>,
}

#[derive(Clone)]
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

impl<'t> PossibleTypes for Union<'t> {
    fn get_possible_types(&self) -> &[&Object] {
        &self.possible_types
    }
}

impl<'t> Fielded<'t> for Union<'t> {
    fn get_fields(&self) -> Vec<&'t Field<'t>> {
        self.possible_types
            .iter()
            .flat_map(|f| f.get_fields())
            .collect::<Vec<_>>()
    }
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

#[derive(Clone)]
pub enum InputType<'t> {
    Object(InputObject<'t>),
    Scalar(Scalar),
}

#[derive(Clone)]
pub enum NullableType<'t, T = NamedType<'t>> {
    Named(&'t T),
    List(Box<Type<'t, T>>),
}

#[derive(Clone)]
pub enum Type<'t, T = NamedType<'t>> {
    Named(&'t T),
    List(Box<Type<'t, T>>),
    Nullable(NullableType<'t, T>),
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

impl<'t> Fielded<'t> for FieldedType<'t> {
    fn get_fields(&'t self) -> Vec<&'t Field<'t>> {
        match self {
            FieldedType::Union(u) => u.get_fields(),
            FieldedType::Object(o) => o.get_fields(),
            FieldedType::Interface(i) => i.get_fields(),
        }
    }
}

pub trait Fielded<'t> {
    fn get_fields(&'t self) -> Vec<&'t Field<'t>>;

    fn get_field(&'t self, name: &str) -> Option<&'t Field> {
        self.get_fields()
            .into_iter()
            .find(|field| field.name.0 == name)
    }
}

pub trait PossibleTypes {
    fn get_possible_types(&self) -> &[&Object];

    fn get_possible_type(&self, name: &str) -> Option<&Object> {
        self.get_possible_types()
            .iter()
            .find(|t| t.name == name)
            .copied()
    }
}

#[derive(Clone)]
pub enum NullableSelectionType<'t> {
    Named(NamedSelectionType<'t>),
    List(Box<ListSelectionType<'t>>),
}

#[derive(Clone)]
pub enum ListSelectionType<'t> {
    Named(NamedSelectionType<'t>),
    Nullable(NullableSelectionType<'t>),
    List(Box<ListSelectionType<'t>>),
}

#[derive(Clone)]
pub enum NamedSelectionType<'t> {
    SelectionSet(SelectionSet<'t>),
    Leaf(&'t LeafType),
}

#[derive(Clone)]
pub enum SelectionType<'t> {
    Named(NamedSelectionType<'t>),
    List(ListSelectionType<'t>),
    Nullable(NullableSelectionType<'t>),
}

#[derive(Clone)]
pub struct FieldSelection<'t> {
    name: SelectionName<'t>,
    of_type: SelectionType<'t>,
}

#[derive(Clone)]
pub struct FragmentSelection<'t>(SelectionSet<'t>);

#[derive(Clone)]
pub enum Selection<'t> {
    Field(FieldSelection<'t>),
    Fragment(FragmentSelection<'t>),
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
    doc: query::NamedOperation,
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
