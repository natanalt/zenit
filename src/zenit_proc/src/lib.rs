use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

pub(crate) mod utils;

mod m_define_node_type;
mod m_ext_repr;
mod m_from_node;
mod m_packed_parser;
mod m_tupled_container_derefs;

/// Implements the [`zenit_lvl::PackedParser`] trait on given type, if all of its fields also
/// implement it.
///
/// If a field is marked with the `#[from(T)]` attribute, it'll be read as `T` (`T` must implement
/// PackedParser as well), and then converted into the field's type using its `From<T>`
/// implementation.
#[proc_macro_derive(PackedParser, attributes(from))]
pub fn packed_parser_derive(input: TokenStream) -> TokenStream {
    m_packed_parser::packed_parser_derive(input)
}

/// Implements the [`zenit_lvl::FromNode`] trait on given type if all of its fields also
/// implement it.
///
/// Each field to be parsed must be marked with one of those attributes:
///  * `#[node("NAME")]` - expects exactly one node of name `NAME` in the parent node. The field's
///    type must implement the [`zenit_lvl::FromNode`] trait as well.
///  * `#[nodes("NAME")]` - expects zero or more nodes of name `NAME` which will be accumualted in
///    this field. The field's type must be convertible using `.try_into()` from a [`Vec`]
///
/// If this structure has any fields not tagged with a node attribute, it must also implement the
/// [`Default`] trait, to provide a default for such fields.
///
/// ## Example
/// ```ignore
// Ignored, since doctests can't link zenit_lvl
/// #[derive(Debug, Clone, Default, zenit_proc::FromNode)]
/// struct ParserTest {
///     /// Expects a single `NAME` node, whose payload will be parsed as a [`CString`] and then
///     /// converted to a [`String`].
///     #[node("NAME")]
///     pub name: String,
///
///     /// Expects zero or more `IDK_` nodes, whose payloads will be put into this vector as
///     /// strings.
///     #[node("IDK_")]
///     pub other_values: Vec<String>,
///     
///     /// Expects exactly 4 `TXTR` nodes containing null terminated strings
///     #[nodes("TXTR")]
///     pub so_fancy: [CString; 4],
///
///     /// As it's untagged, this value will be initialized with its default value, that is `None`,
///     /// by the derived [`zenit_lvl::FromNode::from_node`] function.
///     pub will_be_none: Option<u32>,
/// }
/// ```
///
#[proc_macro_derive(FromNode, attributes(node, nodes))]
pub fn from_node_derive(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    match m_from_node::from_node_derive(derive_input) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

/// Extended `#[repr(T)]` macro, additionally creating [`Into<T>`] and [`TryFrom<T>`]
/// implementations, which should be present in Rust by default, but for some reason aren't.
///
/// **Important:** this macro assumes that `zenit_utils` is available.
///
/// ## Example
// Ignored since the doctests can't link zenit_utils
/// ```ignore
/// use zenit_proc::ext_repr;
///
/// // Actual structure used by Zenit is a bit larger
/// #[ext_repr(u32)]
/// #[derive(Debug)]
/// enum TextureFormat {
///     A8R8G8B8 = 0x15,
///     R5G6B5 = 0x17,
///     A8 = 0x1c,
/// }
///
/// assert_eq!(TextureFormat::try_from(0x17u32), Ok(TextureFormat::R5G6B5));
/// assert_eq!(TextureFormat::A8.into(), 0x1c as u32);
/// ```
///
#[proc_macro_attribute]
pub fn ext_repr(input: TokenStream, source_item: TokenStream) -> TokenStream {
    m_ext_repr::ext_repr(input, source_item)
}

// TODO: think of a better name for "TupledContainerDeref" lol
/// Creates [`std::ops::Deref`] and [`std::ops::DerefMut`] implementations for
/// a struct with a single unnamed (tupled) parameter.
/// 
/// ## Example
/// ```
/// #[derive(zenit_proc::TupledContainerDerefs)]
/// struct ExampleStruct(pub Vec<usize>);
/// 
/// let value = ExampleStruct(vec![100, 200, 300]);
/// assert_eq!(value[1], 200);
/// ```
#[proc_macro_derive(TupledContainerDerefs)]
pub fn tupled_container_derefs(input: TokenStream) -> TokenStream {
    m_tupled_container_derefs::tupled_container_derefs(input)
}

// This is kept in case it's ever useful again (likely not but whatevs)
#[proc_macro]
#[deprecated = "replaced in favor of derive macros"]
pub fn define_node_type(input: TokenStream) -> TokenStream {
    m_define_node_type::define_node_type(input)
}
