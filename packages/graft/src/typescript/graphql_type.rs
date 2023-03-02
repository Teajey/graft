use std::fmt::Write;

use convert_case::{Case, Casing};
use eyre::{eyre, Result};

use super::{possibly_write_description, Typescriptable, TypescriptableWithBuffer, WithContext, TypescriptName};
use crate::gen::Buffer;
use crate::graphql::schema::{NamedType, TypeRef, TypeRefContainer, named_type::{Enum, InputObject, Interface, Object, Scalar, Union}};
use crate::util::{Named, MaybeNamed};

impl TypescriptName for Scalar {
    fn typescript_name(&self) -> String {
        format!("{name}Scalar", name = self.name)
    }
}

impl TypescriptName for Interface {
    fn typescript_name(&self) -> String {
        format!("{name}Interface", name = self.name)
    }
}

impl TypescriptName for Union {
    fn typescript_name(&self) -> String {
        format!("{name}Union", name = self.name)
    }
}

impl TypescriptName for NamedType {
    fn typescript_name(&self) -> String {
        match self {
            NamedType::Scalar(scalar) => {
                scalar.typescript_name()
            }
            NamedType::Interface(interface) => {
                interface.typescript_name()
            }
            NamedType::Union(uni) => {
                uni.typescript_name()
            }
            other_type => other_type.name().to_owned(),
        }
    }
}

impl<'a, 'b, 'c> Typescriptable for WithContext<'a, 'b, 'c, Scalar> {
    fn as_typescript(&self) -> Result<String> {
        let Self { target: scalar, ctx } = self;

        let mut buf = String::new();
        possibly_write_description(&mut buf, scalar.description.as_ref())?;
        let scalar_type = match scalar.name.as_str() {
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
        writeln!(buf, "export type {ts_name} = {scalar_type};", ts_name = scalar.typescript_name())?;
        Ok(buf)
    }
}

impl<'a, 'b, 'c> Typescriptable for WithContext<'a, 'b, 'c, Object> {
    fn as_typescript(&self) -> Result<String> {
        let Self { target: object, ctx } = self;

        let mut buf = String::new();
        possibly_write_description(&mut buf, object.description.as_ref())?;
        write!(buf, "export type {ts_name} = ", ts_name = object.name)?;
        for interface in &object.interfaces {
            let interface = ctx
                .index
                .type_from_ref(interface.clone())?
                .try_into_named()?;
            write!(buf, "{} & ", interface.typescript_name())?;
        }
        writeln!(buf, "{{")?;
        for f in &object.fields {
            possibly_write_description(&mut buf, f.description.as_ref())?;
            writeln!(
                buf,
                "  {}: {},",
                f.name,
                ctx.with(&f.of_type).as_typescript()?
            )?;
        }
        writeln!(buf, "}}")?;
        Ok(buf)
    }
}

impl<'a, 'b, 'c> Typescriptable for WithContext<'a, 'b, 'c, Interface> {
    fn as_typescript(&self) -> Result<String> {
        let Self { target: interface, ctx } = self;

        let mut buf = String::new();
        possibly_write_description(&mut buf, interface.description.as_ref())?;
        write!(buf, "export type {ts_name} = ", ts_name = interface.typescript_name())?;
        for interface in &interface.interfaces {
            let interface = ctx
                .index
                .type_from_ref(interface.clone())?
                .try_into_named()?;
            write!(buf, "{} & ", interface.typescript_name())?;
        }
        writeln!(buf, "{{")?;
        for f in &interface.fields {
            possibly_write_description(&mut buf, f.description.as_ref())?;
            writeln!(
                buf,
                "  {}: {},",
                f.name,
                ctx.with(&f.of_type).as_typescript()?
            )?;
        }
        writeln!(buf, "}}")?;
        Ok(buf)
    }
}

impl Typescriptable for Union {
    fn as_typescript(&self) -> Result<String> {
        let mut buf = String::new();
        possibly_write_description(&mut buf, self.description.as_ref())?;
        let possible_types = self.possible_types
            .iter()
            .map(|t| {
                t.maybe_name()
                    .ok_or_else(|| eyre!("Non-named type cannot be a Union."))
            })
            .collect::<Result<Vec<_>>>()?
            .join(" | ");
        writeln!(
            buf,
            "export type {ts_name} = {possible_types};", ts_name = self.typescript_name(),
        )?;
        Ok(buf)
    }
}

impl Typescriptable for Enum {
    fn as_typescript(&self) -> Result<String> {
        let mut buf = String::new();
        possibly_write_description(&mut buf, self.description.as_ref())?;
        writeln!(buf, "export enum {ts_name} {{", ts_name = self.name)?;
        for v in &self.enum_values {
            possibly_write_description(&mut buf, v.description.as_ref())?;
            writeln!(
                buf,
                "  {} = \"{}\",",
                v.name.to_case(Case::Pascal),
                v.name
            )?;
        }
        writeln!(buf, "}}")?;
        Ok(buf)
    }
}

impl<'a, 'b, 'c> Typescriptable for WithContext<'a, 'b, 'c, InputObject> {
    fn as_typescript(&self) -> Result<String> {
        let Self { target: input_object, ctx } = self;

        let mut buf = String::new();
        possibly_write_description(&mut buf, input_object.description.as_ref())?;
        writeln!(buf, "export type {ts_name} = {{", ts_name = input_object.name)?;
        for f in &input_object.input_fields {
            possibly_write_description(&mut buf, f.description.as_ref())?;
            let is_non_null = matches!(f.of_type, TypeRef::Container(TypeRefContainer::NonNull { .. }));
            writeln!(
                buf,
                "  {}{}: {},",
                f.name,
                if is_non_null {""} else {"?"},
                ctx.with(&f.of_type).as_typescript()?
            )?;
        }
        writeln!(buf, "}}")?;
        Ok(buf)
    }
}


impl<'a, 'b, 'c> TypescriptableWithBuffer for WithContext<'a, 'b, 'c, NamedType> {
    fn as_typescript_on(&self, buffer: &mut Buffer) -> Result<()> {
        let WithContext { target, ctx } = self;

        if target.is_internal() {
            return Ok(());
        }

        match target {
            NamedType::Scalar(scalar) => {
                let scalar_ts = ctx.with(scalar).as_typescript()?;
                write!(buffer.scalars, "{scalar_ts}")?;
            }
            NamedType::Object(object) => {
                let object_ts = ctx.with(object).as_typescript()?;
                write!(buffer.objects, "{object_ts}")?;
            }
            NamedType::Interface(interface) => {
                let interface_ts = ctx.with(interface).as_typescript()?;
                write!(buffer.interfaces, "{interface_ts}")?;
            }
            NamedType::Union(uni) => {
                let union_ts = uni.as_typescript()?;
                write!(buffer.unions, "{union_ts}")?;
            }
            NamedType::Enum(e) => {
                let enum_ts = e.as_typescript()?;
                write!(buffer.enums, "{enum_ts}")?;
            }
            NamedType::InputObject(input_object) => {
                let input_object_ts = ctx.with(input_object).as_typescript()?;
                write!(buffer.input_objects, "{input_object_ts}")?;
            }
        }

        Ok(())
    }
}
