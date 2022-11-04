use std::fmt::Write;

use convert_case::{Case, Casing};
use eyre::{eyre, Result};

use super::{possibly_write_description, Typescriptable, TypescriptableWithBuffer};
use crate::introspection_response::{Type, TypeRef};
use crate::Buffer;

impl<'a> TypescriptableWithBuffer<'a> for Type {
    fn as_typescript_on(&self, buffer: &mut Buffer) -> Result<()> {
        match self {
            Type::Scalar { name, description } => {
                possibly_write_description(&mut buffer.scalars, description.as_ref())?;
                let scalar_type = match name.as_str() {
                    "ID" => r#"NewType<string, "ID">"#,
                    "String" => "string",
                    "Int" => "number",
                    "Float" => "number",
                    "Boolean" => "boolean",
                    _ => "unknown",
                };
                writeln!(buffer.scalars, "export type {name}Scalar = {scalar_type};")?;
            }
            Type::Enum {
                name,
                description,
                enum_values,
            } => {
                if name.starts_with('_') {
                    return Ok(());
                }
                possibly_write_description(&mut buffer.enums, description.as_ref())?;
                writeln!(buffer.enums, "export enum {name} {{")?;
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
            Type::Object {
                name,
                description,
                fields,
                interfaces,
            } => {
                if name.starts_with('_') {
                    return Ok(());
                }
                possibly_write_description(&mut buffer.objects, description.as_ref())?;
                writeln!(buffer.objects, "export type {name} = {{")?;
                for f in fields {
                    possibly_write_description(&mut buffer.objects, f.description.as_ref())?;
                    writeln!(
                        buffer.objects,
                        "  {}: {},",
                        f.name,
                        f.of_type.as_typescript()?
                    )?;
                }
                writeln!(buffer.objects, "}}")?;
            }
            Type::InputObject {
                name,
                description,
                input_fields,
            } => {
                if name.starts_with('_') {
                    return Ok(());
                }
                possibly_write_description(&mut buffer.objects, description.as_ref())?;
                writeln!(buffer.input_objects, "export type {name} = {{")?;
                for f in input_fields {
                    possibly_write_description(&mut buffer.input_objects, f.description.as_ref())?;
                    if let TypeRef::NonNull { .. } = f.of_type {
                        writeln!(
                            buffer.input_objects,
                            "  {}: {},",
                            f.name,
                            f.of_type.as_typescript()?
                        )?;
                    } else {
                        writeln!(
                            buffer.input_objects,
                            "  {}?: {},",
                            f.name,
                            f.of_type.as_typescript()?
                        )?;
                    }
                }
                writeln!(buffer.input_objects, "}}")?;
            }
            Type::Union {
                name,
                description,
                possible_types,
            } => todo!(),
            Type::Interface {
                name,
                description,
                fields,
                possible_types,
            } => todo!(),
            Type::List { .. } => return Err(eyre!("Top-level lists not supported.")),
            Type::NonNull { .. } => return Err(eyre!("Top-level non-nulls not supported.")),
        }

        Ok(())
    }
}
