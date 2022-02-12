
// TODO: cleanup `define_node_type!`

/// An overcomplicated structural node type and parser generator, backed by [`zenit_lvl_proc`]
/// derive and attribute macros.
/// 
/// The core syntax of this macro is:
/// ```ignore
/// define_node_type! {
///     // "name" is the 4-byte node identifier
///     "name" as RootTypeName {
///         // structural definition
///     }
/// }
/// ```
/// 
/// ### Structural definitions
/// Structural definitions define what nodes contain internally in their payload, whether it's
/// more nodes or specific packed data.
/// 
/// The hierarchical variant is as follows
/// ```ignore
/// {
///     // NOTE: mind the commas, the parser is very strict about them
///     
///     // TODO: optional variant for zero or one node instances
///     // something like "abcd" -> field_name: Option<FieldType> { /* optional s.d. */ }
/// 
///     // New structural definition. This will expect a *single* node of ID "exm1", and create a new
///     // type called FieldType containing its fields
///     "exm1" -> field_name1: FieldType { /* structural definition */ }
/// 
///     // New structural definition. This will expect zero or more nodes of given format, defined
///     // within the macro. 
///     "exm2" -> field_name2: Vec<FieldType> { /* structural definition */ }
///     
///     // External definition, expecting a single node of this type.
///     // External types require definition of the [`NodeDeserializer`] trait, implemented by this
///     // macro and for some common types (see [`NodeDeserializer`] docs for details) 
///     "exm3" -> field_name3: FieldType,
/// 
///     // External definition, expecting zero or more nodes of this type
///     "exm4" -> field_name4: Vec<FieldType>,
/// }
/// ```
/// 
/// A structural definition may also define a packed format:
/// ```
/// {
///     // Fields don't have *any* padding and will be parsed sequentially, with multibyte values
///     // read in little endian format
///     field_a: u32,
///     // AnEnumWithReprU32 must implement num_traits::FromPrimitive
///     field_b: AnEnumWithReprU32 as u32,
///     another_field: u8,
/// }
/// ```
#[macro_export]
macro_rules! define_node_type {
    // Packed data node that I could put in node_type_impl! but it works more conveniently over here 
    ($root_name:literal as $root_type:ident {
        $($field_name:ident: $field_type:ty $(as $source_type:ty)?,)*
    }) => {
        #[derive(Debug, Clone, zenit_lvl_proc::PackedParser)]
        pub struct $root_type {
            $(
                $(#[reinterpret($source_type)])?
                pub $field_name: $field_type,
            )*
        }
    };
    
    // Core structural definition
    ($root_name:literal as $root_type:ident { $($inner:tt)* }) => {
        $crate::node_type_impl! {
            $root_type $($inner)* __node_break_line__
        }
    };
}

/// Internal use only
#[macro_export]
macro_rules! node_type_impl {
    // Finisher
    (
        $type_name:ident
        __node_break_line__
        $($child_node_name:literal, $field_name:ident, $field_type:ty,)*
    ) => {
        #[derive(Debug, Clone, zenit_lvl_proc::NodeParser)]
        pub struct $type_name {
            $(
                #[node($child_node_name)]
                pub $field_name: $field_type,
            )*
        }
    };

    // Field with type externally defined
    (
        $type_name:ident
        $child_node_name:literal -> $field_name:ident: $field_type:ty,
        $($tail:tt)*
    ) => {
        $crate::node_type_impl! {
            $type_name
            $($tail)*
            $child_node_name, $field_name, $field_type,
        }
    };

    // Many-field definition with type internally defined
    (
        $type_name:ident
        $child_node_name:literal -> $field_name:ident: Vec < $field_type:ident > {
            $($contents:tt)*
        }
        $($tail:tt)*
    ) => {
        $crate::define_node_type! {
            $child_node_name as $field_type {
                $($contents)*
            }
        }

        $crate::node_type_impl! {
            $type_name
            $($tail)*
            $child_node_name, $field_name, Vec<$field_type>,
        }
    };

    // Single-field definition with type internally defined
    (
        $type_name:ident
        $child_node_name:literal -> $field_name:ident: $field_type:ident {
            $($contents:tt)*
        }
        $($tail:tt)*
    ) => {
        $crate::define_node_type! {
            $child_node_name as $field_type {
                $($contents)*
            }
        }

        $crate::node_type_impl! {
            $type_name
            $($tail)*
            $child_node_name, $field_name, $field_type,
        }
    };
}
