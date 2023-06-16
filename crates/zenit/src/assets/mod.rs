//! Game asset management
//!
//! This includes loading game assets using [`zenit_lvl`] for usage with modules like the renderer,
//! managing their lifetimes, providing any additional necessary functionality.

#[doc(inline)]
pub use asset_loader::*;
mod asset_loader;
#[doc(inline)]
pub use asset_manager::*;
mod asset_manager;
#[doc(inline)]
pub use game_root::*;
mod game_root;

/// Built-in level file. See the `crates/zenit/assets/builtin` directory for details.
pub const ZENIT_BUILTIN_LVL: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/zenit_builtin.lvl"));
