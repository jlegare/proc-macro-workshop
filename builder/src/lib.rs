extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let tree = parse_macro_input!(input as DeriveInput);
    let struct_name = &tree.ident;
    let struct_fields = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
        ..
    }) = tree.data
    {
        named
    } else {
        unimplemented!()
    };

    let builder_name = format_ident!("{}Builder", struct_name);
    let builder_fields = struct_fields.iter().map(optionize_struct_field);
    let builder_methods = struct_fields.iter().map(methodize_struct_field);
    let builder_assignments = struct_fields.iter().map(assign_field);
    let builder_initializations = struct_fields.iter().map(initialize_field);

    let expanded = quote! {
        pub struct #builder_name {
            #(#builder_fields,)*
        }

        impl #builder_name {
            #(#builder_methods)*

            pub fn build(&mut self) -> Result<#struct_name, Box<dyn std::error::Error>> {
                Ok(#struct_name {
                    #(#builder_assignments,)*
                })
            }
        }

        impl #struct_name {
            fn builder() -> #builder_name {
                #builder_name {
                    #(#builder_initializations,)*
                }
            }
        }
    };

    expanded.into()
}

fn assign_field(field: &syn::Field) -> proc_macro2::TokenStream {
    let field_name = &field.ident;
    match unwrap_inner_type(&field.ty, "Option") {
        Some(_) => quote! { #field_name: self.#field_name.clone() },
        None => quote! {
            #field_name: self.#field_name.clone().ok_or(format!("{} is not set.", stringify!(#field_name)))?
        },
    }
}

fn initialize_field(field: &syn::Field) -> proc_macro2::TokenStream {
    let field_name = &field.ident;
    quote! {
        #field_name: None
    }
}

fn optionize_struct_field(field: &syn::Field) -> proc_macro2::TokenStream {
    let field_name = &field.ident;
    let field_type = &field.ty;
    match unwrap_inner_type(field_type, "Option") {
        Some(_) => quote! { #field_name: #field_type },
        None => quote! { #field_name: std::option::Option<#field_type> },
    }
}

fn methodize_struct_field(field: &syn::Field) -> proc_macro2::TokenStream {
    let field_name = &field.ident;
    let field_type = &field.ty;
    match unwrap_inner_type(&field_type, "Option") {
        Some(inner_type) => {
            quote! { pub fn #field_name(&mut self, #field_name: #inner_type) -> &mut Self {
                self.#field_name = Some(#field_name);
                self
            } }
        }
        None => quote! { pub fn #field_name(&mut self, #field_name: #field_type) -> &mut Self {
            self.#field_name = Some(#field_name);
            self
        } },
    }
}

fn unwrap_inner_type<'a>(field_type: &'a syn::Type, wrapper: &str) -> Option<&'a syn::Type> {
    if let syn::Type::Path(syn::TypePath {
        path: syn::Path { segments, .. },
        ..
    }) = field_type
    {
        if let std::option::Option::Some(syn::PathSegment {
            ident,
            arguments:
                syn::PathArguments::AngleBracketed(
                    syn::AngleBracketedGenericArguments { args: types, .. },
                    ..,
                ),
            ..
        }) = segments.first()
        {
            if ident == wrapper {
                if let std::option::Option::Some(syn::GenericArgument::Type(inner_type)) =
                    types.first()
                {
                    return std::option::Option::Some(inner_type);
                }
            }
        }
    }

    None
}
