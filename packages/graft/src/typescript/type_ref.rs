use eyre::Result;

use super::{TypescriptName, Typescriptable, WithContext};
use crate::graphql::schema::{Type, TypeRef, TypeRefContainer};

impl<'a, 'b, 'c> Typescriptable for WithContext<'a, 'b, 'c, TypeRef> {
    fn as_typescript(&self) -> Result<String> {
        recursive_typescriptify(self, &mut true)
    }

    fn as_typescript_non_nullable(&self) -> Result<String> {
        recursive_typescriptify(self, &mut false)
    }
}

fn recursive_typescriptify(
    with_context: &WithContext<'_, '_, '_, TypeRef>,
    nullable: &mut bool,
) -> Result<String> {
    let WithContext { target, ctx } = with_context;
    let this_type = ctx.index.type_from_ref((*target).clone())?;
    let type_name = match this_type {
        Type::Container(TypeRefContainer::NonNull { of_type }) => {
            *nullable = false;
            recursive_typescriptify(&ctx.with(&of_type), nullable)?
        }
        Type::Container(TypeRefContainer::List { of_type }) => {
            let string = recursive_typescriptify(&ctx.with(&of_type), nullable)?;
            format!("{string}[]")
        }
        Type::Named(other_type) => other_type.typescript_name(),
    };

    let type_name = if *nullable {
        format!("Nullable<{type_name}>")
    } else {
        type_name
    };

    Ok(type_name)
}
