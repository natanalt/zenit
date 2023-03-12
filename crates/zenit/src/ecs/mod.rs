//! The Zenit entity component system implementation
//!
//! ## Implementation details
//! For the sake of structural simplicity, I wanted the `Universe` structure to have a Vec for each
//! component inlined in the struct itself. In C++ this would be easy to handle with a std::tuple
//! and a bunch of template parameters, but in Rust I had to resort to writing an unclean macro.
//!
//! I'm not proud of how clean this code is - it can probably be cleaned up, but it works!
//!

use std::{any::Any, num::NonZeroU32};

pub mod components;
pub mod macros;
pub mod accessor;

mod universe;
pub use universe::*;

/// An entity handle. It's very cheap to copy (2x32-bit values)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity {
    pub index: u32,
    pub generation: NonZeroU32,
}

/// Marker trait for components.
///
/// Any new component implementations must be registered in `src/ecs/universe.rs` to be properly used.
pub trait Component: Any {}
