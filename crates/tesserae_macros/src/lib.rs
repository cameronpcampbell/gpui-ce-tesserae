//! Derives used by Tesserae components.

use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{
    Attribute, Data, DeriveInput, Error, ExprClosure, Fields, Generics, Lifetime,
    LitStr, Meta, Token, Type, parse_macro_input,
    punctuated::Punctuated,
    visit_mut::{self, VisitMut},
};

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

/// Implements `tesserae_utils::StylesEnum` for an enum of style refinements.
#[proc_macro_derive(Styles, attributes(class, styles, styles_data))]
pub fn derive_styles(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match expand_styles(input) {
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

fn expand_styles(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;
    let variants = match &input.data {
        Data::Enum(data) => &data.variants,
        _ => {
            return Err(Error::new_spanned(
                name,
                "Styles can only be derived for enums",
            ));
        }
    };

    let styles_lifetime = unique_styles_lifetime(&input.generics);
    let data_type = parse_styles_data(&input.attrs, &styles_lifetime)?;
    let mut class_arms = Vec::with_capacity(variants.len());
    let mut refinement_arms = Vec::with_capacity(variants.len());

    for variant in variants {
        if !matches!(variant.fields, Fields::Unit) {
            return Err(Error::new_spanned(
                &variant.fields,
                "Styles only supports unit variants",
            ));
        }

        let mut styles = None;
        let mut class = None;

        for attribute in &variant.attrs {
            if attribute.path().is_ident("styles") {
                if styles.is_some() {
                    return Err(Error::new_spanned(
                        attribute,
                        "duplicate #[styles(...)] attribute",
                    ));
                }

                let closure = attribute.parse_args::<ExprClosure>()?;

                if closure.inputs.len() != 2 {
                    return Err(Error::new_spanned(
                        &closure,
                        "#[styles(...)] expects a closure with two parameters",
                    ));
                }

                styles = Some(closure);
            } else if attribute.path().is_ident("class") {
                if class.is_some() {
                    return Err(Error::new_spanned(
                        attribute,
                        "duplicate #[class(...)] attribute",
                    ));
                }

                let value = attribute.parse_args::<LitStr>()?;

                if value.value().is_empty() {
                    return Err(Error::new_spanned(
                        value,
                        "class name cannot be empty",
                    ));
                }

                class = Some(value);
            }
        }

        let styles = styles.ok_or_else(|| {
            Error::new_spanned(
                &variant.ident,
                "missing #[styles(|refinement, data| ...)] attribute",
            )
        })?;
        let class = class.unwrap_or_else(|| {
            LitStr::new(
                &variant.ident.to_string().to_case(Case::Snake),
                variant.ident.span(),
            )
        });

        let variant_name = &variant.ident;

        class_arms.push(quote! {
            Self::#variant_name => #class,
        });

        refinement_arms.push(quote! {
            Self::#variant_name => {
                let apply: fn(
                    ::gpui::StyleRefinement,
                    Self::Data<#styles_lifetime>,
                ) -> ::gpui::StyleRefinement = #styles;

                apply(refinement, data)
            }
        });
    }

    let generics = &input.generics;
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics ::tesserae_utils::StylesEnum
            for #name #type_generics #where_clause
        {
            type Data<#styles_lifetime> = #data_type;

            fn class_name(&self) -> &'static str {
                match self {
                    #(#class_arms)*
                }
            }

            fn refine<#styles_lifetime>(
                &self,
                refinement: ::gpui::StyleRefinement,
                data: Self::Data<#styles_lifetime>,
            ) -> ::gpui::StyleRefinement {
                match self {
                    #(#refinement_arms)*
                }
            }
        }
    })
}

fn parse_styles_data(
    attributes: &[Attribute],
    styles_lifetime: &Lifetime,
) -> syn::Result<proc_macro2::TokenStream> {
    let mut styles_data = None;

    for attribute in attributes
        .iter()
        .filter(|attribute| attribute.path().is_ident("styles_data"))
    {
        if styles_data.is_some() {
            return Err(Error::new_spanned(
                attribute,
                "duplicate #[styles_data(...)] attribute",
            ));
        }

        styles_data = Some(
            attribute
                .parse_args_with(Punctuated::<Type, Token![,]>::parse_terminated)?,
        );
    }

    let Some(mut types) = styles_data else {
        return Ok(quote! { () });
    };

    let mut lifetime_elider = ReferenceLifetimeElider(styles_lifetime);
    for data_type in &mut types {
        lifetime_elider.visit_type_mut(data_type);
    }

    Ok(match types.len() {
        0 => quote! { () },
        1 => {
            let data_type = types.first().expect("length checked");

            quote! { #data_type }
        }

        _ => quote! { (#types) },
    })
}

fn unique_styles_lifetime(generics: &Generics) -> Lifetime {
    let existing = generics
        .lifetimes()
        .map(|parameter| parameter.lifetime.ident.to_string())
        .collect::<Vec<_>>();
    let mut name = "__tesserae_styles".to_string();

    while existing.iter().any(|existing| existing == &name) {
        name.push('_');
    }

    Lifetime::new(&format!("'{name}"), Span::call_site())
}

struct ReferenceLifetimeElider<'a>(&'a Lifetime);

impl VisitMut for ReferenceLifetimeElider<'_> {
    fn visit_type_reference_mut(&mut self, reference: &mut syn::TypeReference) {
        let needs_lifetime = match &reference.lifetime {
            None => true,
            Some(lifetime) => lifetime.ident == "_",
        };

        if needs_lifetime {
            reference.lifetime = Some(self.0.clone());
        }

        visit_mut::visit_type_reference_mut(self, reference);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn rejects_non_enums() {
        let input: DeriveInput = parse_quote! {
            struct Invalid;
        };

        assert_eq!(
            expand_styles(input).unwrap_err().to_string(),
            "Styles can only be derived for enums"
        );
    }

    #[test]
    fn rejects_variants_with_fields() {
        let input: DeriveInput = parse_quote! {
            enum Invalid {
                #[styles(|refinement, ()| refinement)]
                Variant(bool),
            }
        };

        assert_eq!(
            expand_styles(input).unwrap_err().to_string(),
            "Styles only supports unit variants"
        );
    }

    #[test]
    fn requires_styles_for_each_variant() {
        let input: DeriveInput = parse_quote! {
            enum Invalid {
                Variant,
            }
        };

        assert_eq!(
            expand_styles(input).unwrap_err().to_string(),
            "missing #[styles(|refinement, data| ...)] attribute"
        );
    }
}
