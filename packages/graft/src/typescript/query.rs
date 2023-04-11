use std::fmt::Write;

use eyre::{eyre, Result};

use crate::{
    gen::Buffer,
    graphql::query::{
        Definition, Fragment, Name, NamedOperation, Operation, OperationType, VariableDefinitions,
    },
    typescript::{Typescriptable, TypescriptableWithBuffer, WithContext},
};

impl<'a, 'b, 'c> Typescriptable for WithContext<'a, 'b, 'c, VariableDefinitions> {
    fn as_typescript(&self) -> Result<String> {
        let Self {
            target: VariableDefinitions(variable_definitions),
            ctx,
        } = self;

        let mut buf = String::new();

        if variable_definitions.is_empty() {
            writeln!(buf, "Record<string, never>;")?;
        } else {
            writeln!(buf, "{{")?;
            for def in variable_definitions {
                let ts_type = ctx.index.type_from_ref(def.of_type.into())?.into_type_ref();
                writeln!(
                    buf,
                    "  {}{}: {},",
                    def.variable.name,
                    if ts_type.is_non_null() { "" } else { "?" },
                    ctx.with(&ts_type).as_typescript()?
                )?;
            }
            writeln!(buf, "}}")?;
        }

        Ok(buf)
    }
}

impl<'a, 'b, 'c> TypescriptableWithBuffer for WithContext<'a, 'b, 'c, NamedOperation> {
    fn as_typescript_on(&self, buffer: &mut Buffer) -> Result<()> {
        let Self {
            target: operation,
            ctx,
        } = self;

        let (operation_type_name, operation_buffer) = match operation.operation {
            OperationType::Query => ("Query", &mut buffer.queries),
            OperationType::Mutation => ("Mutation", &mut buffer.mutations),
            OperationType::Subscription => ("Subscription", &mut buffer.subscriptions),
        };

        let Some(Name(operation_name)) = &operation.name else {
          return Err(eyre!("Won't typescriptify an unnamed operation."))
        };

        let document_json = serde_json::to_string(operation)?;

        let document_operation_type_name = if ctx.options.documents_hide_operation_name {
            ""
        } else {
            operation_type_name
        };

        let document_name = format!("{operation_name}{document_operation_type_name}Document");

        let args_name = format!(
            "{operation_name}{document_operation_type_name}{arguments_suffix}",
            arguments_suffix = ctx.options.arguments_suffix
        );
        let selection_set_name = format!(
            "{operation_name}{operation_type_name}{selection_set_suffix}",
            selection_set_suffix = ctx.options.selection_set_suffix
        );

        writeln!(
            operation_buffer,
            "export const {document_name} = {document_json} as unknown as {document_type_name}<{selection_set_name}, {args_name}>;",
            document_type_name = ctx.options.document_import.type_name()
        )?;

        writeln!(
            buffer.args,
            "export type {args_name} = {}",
            ctx.with(&operation.variable_definitions).as_typescript()?
        );

        // if operation.variable_definitions.is_empty() {
        //     writeln!(
        //         buffer.args,
        //         "export type {args_name} = Record<string, never>;"
        //     )?;
        // } else {
        //     writeln!(buffer.args, "export type {args_name} = {{")?;
        //     for def in &operation.variable_definitions {
        //         let ts_type = ctx.index.;
        //         writeln!(
        //             buffer.args,
        //             "  {}{}: {},",
        //             def.variable.name,
        //             if ts_type.is_non_null() { "" } else { "?" },
        //             ctx.with(&ts_type).as_typescript()?
        //         )?;
        //     }
        //     writeln!(buffer.args, "}}")?;
        // }

        // operation.selection_set

        Ok(())
    }
}

impl<'a, 'b, 'c> TypescriptableWithBuffer for WithContext<'a, 'b, 'c, Operation> {
    fn as_typescript_on(&self, buffer: &mut Buffer) -> Result<()> {
        let Self {
            target: operation,
            ctx,
        } = self;

        match operation {
            Operation::SelectionSet(_) => todo!(),
            Operation::NamedOperation(named_operation) => {
                ctx.with(named_operation).as_typescript_on(buffer)?;
            }
        }

        Ok(())
    }
}

impl<'a, 'b, 'c> TypescriptableWithBuffer for WithContext<'a, 'b, 'c, Fragment> {
    fn as_typescript_on(&self, buffer: &mut Buffer) -> Result<()> {
        todo!()
    }
}

impl<'a, 'b, 'c> TypescriptableWithBuffer for WithContext<'a, 'b, 'c, Definition> {
    fn as_typescript_on(&self, buffer: &mut Buffer) -> Result<()> {
        todo!()
    }
}
