//! Derive fluent, consuming setters for structs with named fields.

use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Error, Fields, Meta, parse_macro_input};

/// Generates one consuming setter for every named field not marked with
/// `#[nosetter]`.
///
/// Each setter is public and accepts any value implementing [`Into`] for the
/// field's type.
#[proc_macro_derive(BuildSetters, attributes(nosetter))]
pub fn derive_build_setters(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match expand_build_setters(input) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.into_compile_error().into(),
    }
}

fn expand_build_setters(
    input: DeriveInput,
) -> syn::Result<proc_macro2::TokenStream> {
    let name = input.ident;
    let fields = match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => fields.named,
            _ => {
                return Err(Error::new_spanned(
                    name,
                    "BuildSetters can only be derived for structs with named fields",
                ));
            }
        },
        _ => {
            return Err(Error::new_spanned(
                name,
                "BuildSetters can only be derived for structs",
            ));
        }
    };

    let mut setters = Vec::new();
    for field in fields {
        let mut skip = false;
        for attribute in field
            .attrs
            .iter()
            .filter(|attribute| attribute.path().is_ident("nosetter"))
        {
            if !matches!(attribute.meta, Meta::Path(_)) {
                return Err(Error::new_spanned(attribute, "expected #[nosetter]"));
            }
            skip = true;
        }

        if skip {
            continue;
        }

        let field_name = field.ident.expect("named fields have identifiers");
        let field_type = field.ty;
        setters.push(quote! {
            pub fn #field_name(self, value: impl ::core::convert::Into<#field_type>) -> Self {
                Self {
                    #field_name: value.into(),
                    ..self
                }
            }
        });
    }

    let generics = input.generics;
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics #name #type_generics #where_clause {
            #(#setters)*
        }
    })
}
