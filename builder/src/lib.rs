extern crate proc_macro;

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Builder, attributes(builder,))]
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
    let builder_fields = struct_fields.iter().map(optionize_field);
    let builder_methods = struct_fields.iter().map(methodize_field);
    let builder_assignments = struct_fields.iter().map(assign_field);
    let builder_initializations = struct_fields.iter().map(initialize_field);

    let expanded = quote! {
        pub struct #builder_name {
            #(#builder_fields,)*
        }

        impl #builder_name {
            #(#builder_methods)*

            pub fn build(&mut self) -> std::result::Result<#struct_name, std::boxed::Box<dyn std::error::Error>> {
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
    match extract_inner_type(&field.ty, "Option") {
        std::option::Option::Some(_) => quote! { #field_name: self.#field_name.clone() },
        std::option::Option::None => quote! {
            #field_name: self.#field_name.clone().ok_or(format!("{} is not set.", stringify!(#field_name)))?
        },
    }
}

fn extract_attribute<'a>(
    attribute_name: &str,
    attributes: &'a std::vec::Vec<syn::Attribute>,
) -> std::option::Option<&'a syn::Attribute> {
    attributes.iter().find(|&a| {
        let syn::Attribute {
            path: syn::Path { segments, .. },
            ..
        } = a;

        segments
            .iter()
            .find(|&segment| {
                let syn::PathSegment { ident, .. } = segment;

                ident.to_string() == attribute_name
            })
            .is_some()
    })
}

fn extract_inner_type<'a>(
    field_type: &'a syn::Type,
    wrapper: &str,
) -> std::option::Option<&'a syn::Type> {
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

    std::option::Option::None
}

fn extract_property_value(
    property_name: &str,
    attribute: &syn::Attribute,
) -> std::option::Option<std::string::String> {
    match attribute.parse_meta() {
        Ok(syn::Meta::List(syn::MetaList { nested, .. })) => {
            match nested.into_iter().find(|item| {
                if let syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue {
                    path,
                    ..
                })) = item
                {
                    path.is_ident(property_name)
                } else {
                    false
                }
            }) {
                std::option::Option::Some(syn::NestedMeta::Meta(syn::Meta::NameValue(
                    syn::MetaNameValue {
                        lit: syn::Lit::Str(literal),
                        ..
                    },
                ))) => std::option::Option::Some(literal.value()),
                _ => std::option::Option::None,
            }
        }
        _ => std::option::Option::None,
    }
}

fn extract_field_builder<'a>(
    attributes: &'a std::vec::Vec<syn::Attribute>,
) -> Option<proc_macro2::Ident> {
    extract_attribute("builder", &attributes)
        .map(|attribute| {
            extract_property_value("each", attribute)
                .map(|builder_name| format_ident!("{}", builder_name))
        })
        .flatten()
}

fn initialize_field(field: &syn::Field) -> proc_macro2::TokenStream {
    let field_name = &field.ident;
    if extract_inner_type(&field.ty, "Vec").is_some() {
        quote! { #field_name: std::option::Option::Some(vec![]) }
    } else {
        quote! { #field_name: std::option::Option::None }
    }
}

fn methodize_field(field: &syn::Field) -> proc_macro2::TokenStream {
    let field_name = format_ident!("{}", field.ident.as_ref().unwrap().to_string());
    let field_type = &field.ty;

    let field_builder = extract_field_builder(&field.attrs);
    let has_field_builder = field_builder.is_some();
    let field_builder_name = field_builder.unwrap_or_else(|| field_name.clone());

    match extract_inner_type(&field_type, "Option") {
        std::option::Option::Some(inner_type) => {
            quote! { pub fn #field_builder_name(&mut self, #field_name: #inner_type) -> &mut Self {
                self.#field_name = std::option::Option::Some(#field_name);
                self
            } }
        }
        std::option::Option::None => {
            if has_field_builder {
                let inner_type = extract_inner_type(&field_type, "Vec").unwrap();

                quote! { pub fn #field_builder_name(&mut self, #field_name: #inner_type) -> &mut Self {
                    if let std::option::Option::Some(ref mut inner) = self.#field_name {
                        inner.push(#field_name)
                    } else {
                        self.#field_name = std::option::Option::Some(vec![#field_name])
                    };
                    self
                } }
            } else {
                quote! { pub fn #field_builder_name(&mut self, #field_name: #field_type) -> &mut Self {
                    self.#field_name = std::option::Option::Some(#field_name);
                    self
                } }
            }
        }
    }
}

fn optionize_field(field: &syn::Field) -> proc_macro2::TokenStream {
    let field_name = &field.ident;
    let field_type = &field.ty;
    match extract_inner_type(field_type, "Option") {
        std::option::Option::Some(_) => quote! { #field_name: #field_type },
        std::option::Option::None => quote! { #field_name: std::option::Option<#field_type> },
    }
}
