use proc_macro::TokenStream;
use quote::{format_ident, quote};

#[proc_macro_derive(Kind)]
pub fn kind_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);

    let name = &ast.ident;

    let visitor_name = format_ident!("{}Visitor", name);

    let gen = quote! {
        impl crate::graphql::kind::Kind for #name {}
        struct #visitor_name;
        impl<'de> serde::de::Visitor<'de> for #visitor_name {
            type Value = #name;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(
                    f,
                    "a map {{ kind: \"{}\", value: <some name> }}",
                    stringify!(#name)
                )
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let value = crate::graphql::kind::visit_map("\"kind\" must be string \"#name\"", map)?;
                Ok(#name(value))
            }
        }
        impl<'de> Deserialize<'de> for #name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                deserializer.deserialize_map(#visitor_name)
            }
        }
    };
    gen.into()
}
