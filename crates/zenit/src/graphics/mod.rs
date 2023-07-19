//! Zenit graphics renderer
//!
//! The Zenit renderer provides all facilities related to 2D and 3D graphics within the engine.
//! Its design specifically aims to be as simple as possible to use (hence why it underwent several
//! big redesigns early on).
//!
//! The module is split into two main components, described below.
//!
//! ## [`system::RenderSystem`]
//! The render system is an engine [`crate::engine::System`]. It handles actually asking wgpu to
//! render things. At the end of every frame, it collects a snapshot of the requested frame from
//! the ECS.
//!
//! ## [`Renderer`]
//! The renderer is the API entrypoint for creating render resources, like cameras, textures, models,
//! etc. Resources are accessed via dedicated handles, which point to a shared reference count. Once
//! a handle loses its last reference, the underlying resource is removed from the renderer.
//!
//! ## Loading assets
//! The renderer doesn't handle actually loading images, models, or any other visual resources.
//! Whenever a new resource is allocated through the graphics API, its data must be provided
//! directly, for example models are given raw vertex and index data, textures are provided their
//! mipmaps in the dedicated format, and so on.
//!
//! Code that actually parses serialized asset files is located in the [`crate::assets`] module.
//!
//! ## How to render a scene
//! The renderer fetches 3D scene data from the ECS. Here's an example workflow of creating a scene
//! that will be detected by the render system:
//!
//! 1. Create an entity with a [`SceneComponent`].
//!    - This component defines global settings of the scene, such as the skybox, fog settings, etc.
//! 2. Create render entities with [`RenderComponent`]s and attach them to the scene.
//!    - This component stores an entity handle to the parent scene.
//!    - Render entities can then be attached components like `TransformComponent`, `ModelComponent`.
//! 3. Create a camera with a [`CameraComponent`] and a `TransformComponent`.
//!    - This component links a scene entity and the underlying camera handle.
//!    - Multiple cameras can render the same scene.
//!

use wgpu::TextureFormat;

pub mod imgui_renderer;
pub mod macros;
pub mod system;

#[doc(inline)]
pub use resources::*;
mod resources;

#[doc(inline)]
pub use interface::*;
mod interface;

#[doc(inline)]
pub use scene_builder::*;
mod scene_builder;

#[doc(inline)]
pub use device_context::*;
mod device_context;

// TODO: make the RENDER_FORMAT dynamically chosen?

/// Format commonly used for render textures. This is the format to be used by render pipelines
/// and anything else that depends on a constant TextureFormat.
///
/// Currently it defaults to a 16-bit float format to enable HDR support.
pub const RENDER_FORMAT: TextureFormat = TextureFormat::Rgba16Float;
pub const DEPTH_FORMAT: TextureFormat = TextureFormat::Depth24Plus;

/// Various feature flags of the renderer, determined at runtime from the engine's settings and the
/// user's hardware configuration.
pub struct RenderCapabilities {
    /// Specifies whether BC/DXT compression is allowed.
    pub allow_bc_compression: bool,
}
