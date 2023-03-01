use std::fmt::Write;

use convert_case::{Case, Casing};
use eyre::{eyre, Result};

use super::{possibly_write_description, Typescriptable, TypescriptableWithBuffer, WithContext};
use crate::gen::Buffer;
use crate::graphql::schema::{NamedType, TypeRef, TypeRefContainer};
use crate::util::{MaybeNamed, Named};

impl NamedType {
    pub fn typescript_name(&self) -> String {
        match self {
            NamedType::Scalar { name, .. } => {
                format!("{name}Scalar")
            }
            NamedType::Interface { name, .. } => {
                format!("{name}Interface")
            }
            NamedType::Union { name, .. } => {
                format!("{name}Union")
            }
            other_type => other_type.name().to_owned(),
        }
    }
}

impl<'a, 'b, 'c> TypescriptableWithBuffer for WithContext<'a, 'b, 'c, NamedType> {
    fn as_typescript_on(&self, buffer: &mut Buffer) -> Result<()> {
        let WithContext { target, ctx } = self;

        let ts_name = target.typescript_name();
        if target.is_internal() {
            return Ok(());
        }
        match target {
            NamedType::Scalar { name, description } => {
                possibly_write_description(&mut buffer.scalars, description.as_ref())?;
                let scalar_type = match name.as_str() {
                    "ID" => r#"NewType<string, "ID">"#.to_owned(),
                    "String" => "string".to_owned(),
                    "Int" | "Float" => "number".to_owned(),
                    "Boolean" => "boolean".to_owned(),
                    name => {
                        let default = format!(r#"NewType<unknown, "{name}">"#);
                        match &ctx.options.scalar_newtypes {
                            None => default,
                            Some(scalar_newtypes) => {
                                scalar_newtypes.get(name).cloned().unwrap_or(default)
                            }
                        }
                    }
                };
                writeln!(buffer.scalars, "export type {ts_name} = {scalar_type};")?;
            }
            NamedType::Object {
                name: _,
                description,
                fields,
                interfaces,
            } => {
                possibly_write_description(&mut buffer.objects, description.as_ref())?;
                write!(buffer.objects, "export type {ts_name} = ")?;
                for interface in interfaces {
                    let interface = ctx
                        .index
                        .type_from_ref(interface.clone())?
                        .try_into_named()?;
                    write!(buffer.objects, "{} & ", interface.typescript_name())?;
                }
                writeln!(buffer.objects, "{{")?;
                for f in fields {
                    possibly_write_description(&mut buffer.objects, f.description.as_ref())?;
                    writeln!(
                        buffer.objects,
                        "  {}: {},",
                        f.name,
                        ctx.with(&f.of_type).as_typescript()?
                    )?;
                }
                writeln!(buffer.objects, "}}")?;
            }
            NamedType::Interface {
                name: _,
                description,
                fields,
                // I don't think this would be very useful here
                possible_types: _,
                interfaces,
            } => {
                possibly_write_description(&mut buffer.interfaces, description.as_ref())?;
                write!(buffer.interfaces, "export type {ts_name} = ")?;
                for interface in interfaces {
                    let interface = ctx
                        .index
                        .type_from_ref(interface.clone())?
                        .try_into_named()?;
                    write!(buffer.interfaces, "{} & ", interface.typescript_name())?;
                }
                writeln!(buffer.interfaces, "{{")?;
                for f in fields {
                    possibly_write_description(&mut buffer.interfaces, f.description.as_ref())?;
                    writeln!(
                        buffer.interfaces,
                        "  {}: {},",
                        f.name,
                        ctx.with(&f.of_type).as_typescript()?
                    )?;
                }
                writeln!(buffer.interfaces, "}}")?;
            }
            NamedType::Union {
                name: _,
                description,
                possible_types,
            } => {
                possibly_write_description(&mut buffer.interfaces, description.as_ref())?;
                let possible_types = possible_types
                    .iter()
                    .map(|t| {
                        t.maybe_name()
                            .ok_or_else(|| eyre!("Non-named type cannot be a Union."))
                    })
                    .collect::<Result<Vec<_>>>()?
                    .join(" | ");
                writeln!(
                    buffer.unions,
                    "export type {ts_name} = {possible_types};",
                )?;
            }
            NamedType::Enum {
                name: _,
                description,
                enum_values,
            } => {
                possibly_write_description(&mut buffer.enums, description.as_ref())?;
                writeln!(buffer.enums, "export enum {ts_name} {{")?;
                for v in enum_values {
                    possibly_write_description(&mut buffer.enums, v.description.as_ref())?;
                    writeln!(
                        buffer.enums,
                        "  {} = \"{}\",",
                        v.name.to_case(Case::Pascal),
                        v.name
                    )?;
                }
                writeln!(buffer.enums, "}}")?;
            }
            NamedType::InputObject {
                name: _,
                description,
                input_fields,
            } => {
                possibly_write_description(&mut buffer.objects, description.as_ref())?;
                writeln!(buffer.input_objects, "export type {ts_name} = {{")?;
                for f in input_fields {
                    possibly_write_description(&mut buffer.input_objects, f.description.as_ref())?;
                    if let TypeRef::Container(TypeRefContainer::NonNull { .. }) = f.of_type {
                        writeln!(
                            buffer.input_objects,
                            "  {}: {},",
                            f.name,
                            ctx.with(&f.of_type).as_typescript()?
                        )?;
                    } else {
                        writeln!(
                            buffer.input_objects,
                            "  {}?: {},",
                            f.name,
                            ctx.with(&f.of_type).as_typescript()?
                        )?;
                    }
                }
                writeln!(buffer.input_objects, "}}")?;
            }
        }

        Ok(())
    }
}
