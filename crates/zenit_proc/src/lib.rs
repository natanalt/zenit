use proc_macro::TokenStream;

mod m_ext_repr;
mod m_packed_data;

/// Implements the [`zenit_utils::PackedData`] trait on given type.
///
/// There are two paths that this macro takes:
///  * **struct** - All fields must implement `PackedData`. The generated implementation reads and
///    writes struct fields in order of definition.
///  * **enum** - Enums can use `#[parse_as(T)]` alongside this derive macro, can cause the
///    implementation to read/write `T`, and then convert it to `Self`. `T` has to implement
///    `PackedData`, and necessary `TryFrom` and `Into` traits (can be used alongside `ext_repr`)
///
/// *(Note, at the moment tuple structs are not supported)*
#[proc_macro_derive(PackedData, attributes(parse_as))]
pub fn packed_parser_derive(input: TokenStream) -> TokenStream {
    m_packed_data::packed_data_derive(input)
}

/// Extended `#[repr(T)]` macro. Aside from invoking normal `#[repr(T)], it creates the following
/// trait implementations:
///  * [`Into<T>`] for converting from self to repr type
///  * [`TryFrom<T>`] for converting from repr type to self
///  * [`Into<&str>`] for converting into the variant's name
///  * [`TryFrom<&str>`] for converting from the variant's name
///
/// **Note:** The macro assumes that `zenit_utils` is present and usable.
/// 
/// ## Example
/// ```norun
/// use zenit_proc::ext_repr;
///
/// #[ext_repr(u32)]
/// #[derive(Debug, PartialEq, Eq)]
/// enum TextureFormat {
///     A8R8G8B8 = 0x15,
///     R5G6B5 = 0x17,
///     A8 = 0x1c,
/// }
///
///
/// assert_eq!(TextureFormat::try_from(0x17u32), Ok(TextureFormat::R5G6B5));
/// let a8_numeric: u32 = TextureFormat::A8.into();
/// assert_eq!(a8_numeric, 0x1c as u32);
///
/// assert_eq!(TextureFormat::try_from("R5G6B5"), Ok(TextureFormat::R5G6B5));
/// let a8_string: &'static str = TextureFormat::A8.into();
/// assert_eq!(a8_string, "A8");
/// ```
///
#[proc_macro_attribute]
pub fn ext_repr(input: TokenStream, source_item: TokenStream) -> TokenStream {
    m_ext_repr::ext_repr(input, source_item)
}
