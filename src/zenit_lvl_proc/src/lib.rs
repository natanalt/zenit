use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Data};

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
            let field_ty = field.ty.clone();
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
                .map_or(field.ty, |attribute| {
                    attribute
                        .parse_args::<syn::Type>()
                        .expect("expected a valid type")
                });
            
            quote! {
                //#name: <#data_ty as zenit_lvl::node::parser::PackedParser>::parse_packed(r)? as #field_ty,
            }
        })
        .collect::<TokenStream2>();

    quote! {
        impl zenit_lvl::node::parser::PackedParser for #name {
            fn parse_packed<R: std::io::Read>(
                r: &mut R,
            ) -> anyhow::Result<Self> {
                todo!()
            }
        }
    }.into()
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
    }.into()
}
