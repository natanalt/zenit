use proc_macro::TokenStream;

#[proc_macro_derive(PackedParser, attributes(reinterpret))]
pub fn packed_parser_derive(input: TokenStream) -> TokenStream {
    #[path = "packed_parser.rs"]
    mod packed_parser_impl;
    packed_parser_impl::packed_parser_derive(input)
}

#[proc_macro_derive(NodeParser, attributes(node))]
pub fn node_parser_derive(input: TokenStream) -> TokenStream {
    #[path = "node_parser.rs"]
    mod node_parser_impl;
    node_parser_impl::node_parser_derive(input)
}

/// Extended #[repr(Type)] macro, additionally creating Into<Type> and TryFrom<Type> implementations,
/// which should be present in Rust by default.
#[proc_macro_attribute]
pub fn ext_repr(input: TokenStream, source_item: TokenStream) -> TokenStream {
    #[path = "ext_repr.rs"]
    mod ext_repr_impl;
    ext_repr_impl::ext_repr(input, source_item)
}
