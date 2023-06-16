//! The Zenit entity component system implementation
//!
//! ## Implementation details
//! For the sake of structural simplicity, I wanted the `Universe` structure to have a Vec for each
//! component inlined in the struct itself. In C++ this would be easy to handle with a std::tuple
//! and a bunch of variadic template parameters, but in Rust I had to resort to writing an unclean
//! macro.
//!
//! I'm not proud of how unclean this code is - it can probably be cleaned up, but it works!
//!

use std::{any::Any, num::NonZeroU32};

pub mod components;

#[doc(inline)]
pub use builder::*;
mod builder;

#[doc(inline)]
pub use accessor::*;
mod accessor;

#[doc(inline)]
pub use universe::*;
mod universe;

/// An entity handle. It's very cheap to copy (2x32-bit values)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity {
    /// The entity's index within the universe entity set.
    pub index: u32,
    /// The entity's generation number. It's unique across the entire universe.
    pub generation: NonZeroU32,
}

/// Marker trait for components.
pub trait Component: Any + Send + Sync {}
