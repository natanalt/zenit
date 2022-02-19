use proc_macro::TokenStream;

mod m_define_node_type;
mod m_ext_repr;
mod m_node_parser;
mod m_packed_parser;

#[proc_macro_derive(PackedParser, attributes(reinterpret))]
pub fn packed_parser_derive(input: TokenStream) -> TokenStream {
    m_packed_parser::packed_parser_derive(input)
}

#[proc_macro_derive(NodeParser, attributes(node))]
pub fn node_parser_derive(input: TokenStream) -> TokenStream {
    m_node_parser::node_parser_derive(input)
}

/// Extended `#[repr(T)]` macro, additionally creating [`Into<T>`] and [`TryFrom<T>`]
/// implementations, which should be present in Rust by default, but for some reason aren't
#[proc_macro_attribute]
pub fn ext_repr(input: TokenStream, source_item: TokenStream) -> TokenStream {
    m_ext_repr::ext_repr(input, source_item)
}

#[proc_macro]
pub fn define_node_type(input: TokenStream) -> TokenStream {
    m_define_node_type::define_node_type(input)
}
