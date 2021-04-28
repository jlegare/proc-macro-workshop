extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let tree = parse_macro_input!(input as DeriveInput);
    let struct_name = &tree.ident;
    let builder_name = syn::Ident::new(
        &format!("{}Builder", struct_name).to_string(),
        struct_name.span(),
    );

    let struct_fields = if let syn::Data::Struct(syn::DataStruct {
        fields: syn::Fields::Named(syn::FieldsNamed { ref named, .. }),
        ..
    }) = tree.data
    {
        named
    } else {
        unimplemented!()
    };

    let builder_fields = struct_fields.iter().map(optionize_struct_field);
    let builder_methods = struct_fields.iter().map(methodize_struct_field);

    let expanded = quote! {
        pub struct #builder_name {
            #(#builder_fields,)*
        }

        impl CommandBuilder {
            #(#builder_methods)*

            pub fn build(&mut self) -> Result<#struct_name, Box<dyn std::error::Error>> {
                Ok(#struct_name {
                    args: self.args.clone().ok_or("args is not set.")?,
                    current_dir: self.current_dir.clone().ok_or("current_dir is not set.")?,
                    env: self.env.clone().ok_or("env is not set.")?,
                    executable: self.executable.clone().ok_or("executable is not set.")?,
                })
            }
        }

        impl #struct_name {
            fn builder() -> #builder_name {
                #builder_name {
                    executable: None,
                    args: None,
                    env: None,
                    current_dir: None,
                }
            }
        }
    };

    expanded.into()
}

fn optionize_struct_field(
    field: &syn::Field,
) -> proc_macro2::TokenStream {
    let field_name = &field.ident;
    let field_type = &field.ty;
    quote! { #field_name: std::option::Option<#field_type> }
}


fn methodize_struct_field(
    field: &syn::Field,
) -> proc_macro2::TokenStream {
    let field_name = &field.ident;
    let field_type = &field.ty;
    quote! { pub fn #field_name(&mut self, #field_name: #field_type) -> &mut Self {
        self.#field_name = Some(#field_name);
        self
    } }
}
