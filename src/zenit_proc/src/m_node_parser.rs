use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

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
