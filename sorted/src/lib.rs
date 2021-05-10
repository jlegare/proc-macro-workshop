extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn sorted(args: TokenStream, input: TokenStream) -> TokenStream {
    let _ = args;
    let tree = parse_macro_input!(input as syn::DeriveInput);

    parse(tree).unwrap_or_else(|e| e.to_compile_error().into())
}

fn parse(tree: syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    match &tree.data {
        syn::Data::Enum(data) => {
            check_sorted(&data.variants)?;
            Ok((quote! { #tree }).into())
        }
        _ => Err(syn::parse::Error::new(
            proc_macro2::Span::call_site(),
            "expected enum or match expression",
        )),
    }
}

fn check_sorted(
    variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>,
) -> Result<(), syn::Error> {
    for outer in variants.iter() {
        for inner in variants.iter() {
            if outer.ident == inner.ident {
                break;
            } else if outer.ident < inner.ident {
                return Err(syn::parse::Error::new(
                    outer.ident.span(),
                    format!("{} should sort before {}", outer.ident, inner.ident),
                ));
            }
        }
    }

    Ok(())
}
