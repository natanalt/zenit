use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Ident, ItemEnum, Type};

#[proc_macro_derive(PackedParser, attributes(reinterpret))]
pub fn packed_parser_derive(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as DeriveInput);
    let name = parsed.ident;
    let data = match parsed.data {
        Data::Struct(s) => s,
        _ => panic!("expected a struct"),
    };

    let initializers = data
        .fields
        .into_iter()
        .map(|field| {
            let name = field.ident.expect("expected valid field name");
            let field_ty = field.ty;
            let data_ty = field
                .attrs
                .iter()
                .find(|attribute| {
                    attribute
                        .path
                        .get_ident()
                        .map(|i| i.to_string() == "reinterpret")
                        .unwrap_or(false)
                })
                .map_or(field_ty.clone(), |attribute| {
                    attribute
                        .parse_args::<syn::Type>()
                        .expect("expected a valid type")
                });

            quote! {
                #name: #field_ty ::try_from(#data_ty ::parse_packed(r)?)? ,
            }
        })
        .collect::<TokenStream2>();

    quote! {
        impl zenit_utils::packed::PackedParser for #name {
            fn parse_packed<R: std::io::Read>(
                r: &mut R,
            ) -> anyhow::Result<Self> {
                Ok(Self {
                    #initializers
                })
            }
        }
    }
    .into()
}

#[proc_macro_derive(NodeParser, attributes(node))]
pub fn node_parser_derive(input: TokenStream) -> TokenStream {
    let parsed = parse_macro_input!(input as DeriveInput);
    let name = parsed.ident;

    quote! {
        impl zenit_lvl::node::parser::NodeParser for #name {
            fn parse<R: std::io::Read + std::io::Seek>(
                root: zenit_lvl::node::LevelNode,
                r: &mut R,
            ) -> anyhow::Result<Self> {
                todo!()
            }
        }
    }
    .into()
}

/// Extended #[repr(Type)] macro, additionally creating Into<Type> and TryFrom<Type> implementations,
/// which should be present in Rust by default.
#[proc_macro_attribute]
pub fn ext_repr(input: TokenStream, source_item: TokenStream) -> TokenStream {
    let source_item_ts2 = TokenStream2::from(source_item.clone());
    let source_item_cloned = source_item.clone();
    let item = parse_macro_input!(source_item_cloned as ItemEnum);
    let enum_name = item.ident;
    let target_type = parse_macro_input!(input as Type);

    let impl_try_from = {
        let const_decls = item
            .variants
            .iter()
            .map(|variant| {
                let big_name = Ident::new(
                    &format!("{}_value", variant.ident.to_string()),
                    Span::call_site(),
                );
                let name = &variant.ident;
                quote! {
                    const #big_name : #target_type = #enum_name :: #name as #target_type ;
                }
            })
            .collect::<TokenStream2>();

        let match_arms = item
            .variants
            .into_iter()
            .map(|variant| {
                let big_name = Ident::new(
                    &format!("{}_value", variant.ident.to_string()),
                    Span::call_site(),
                );
                let name = &variant.ident;
                quote! {
                    #big_name => Ok(#enum_name :: #name),
                }
            })
            .collect::<TokenStream2>();

        quote! {
            impl TryFrom<#target_type> for #enum_name {
                type Error = zenit_utils::EnumParseError;
                fn try_from(value: #target_type) -> Result<Self, zenit_utils::EnumParseError> {
                    #const_decls
                    match value {
                        #match_arms
                        _ => Err(zenit_utils::EnumParseError::InvalidInput),
                    }
                }
            }
        }
    };

    let impl_into = quote! {
        impl Into<#target_type> for #enum_name {
            fn into(self) -> #target_type {
                self as #target_type
            }
        }
    };

    quote! {
        #[repr(#target_type)]
        #source_item_ts2
        #impl_try_from
        #impl_into
    }
    .into()
}
