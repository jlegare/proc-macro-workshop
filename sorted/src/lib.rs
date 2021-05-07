extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn sorted(args: TokenStream, input: TokenStream) -> TokenStream {
    let _ = args;
    let tree = parse_macro_input!(input as syn::DeriveInput);

    parse_sorted(tree).unwrap_or_else(|e| e.to_compile_error().into())
}

fn parse_sorted(tree: syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    match tree.data {
        syn::Data::Enum(_) => Ok((quote! { #tree }).into()),
        _ => Err(syn::parse::Error::new(
            proc_macro2::Span::call_site(),
            "expected enum or match expression",
        )),
    }
}
