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

use parking_lot::Mutex;
use paste::paste;
use std::sync::Arc;
use wgpu::{Adapter, Device, Instance, Queue, TextureFormat};
use zenit_utils::ArcPool;

pub mod egui_support;
mod frame_state;
pub mod macros;
pub mod system;

#[doc(inline)]
pub use resources::*;
mod resources;

#[doc(inline)]
pub use scene::*;

use self::system::EguiOutput;
mod scene;

// TODO: make the RENDER_FORMAT dynamically chosen?

/// Format commonly used for render textures. This is the format to be used by render pipelines
/// and anything else that depends on a constant TextureFormat.
///
/// Currently it defaults to a 16-bit float format to enable HDR support.
pub const RENDER_FORMAT: TextureFormat = TextureFormat::Rgba16Float;

/// The device context contains public information regarding the current [`wgpu`] instance,
/// including the device, queue, adapter, main surface, etc.
///
/// ## Storage notes
/// The [`DeviceContext`] is stored within the [`Renderer`] and [`system::RenderSystem`] via a
/// shared [`Arc`] reference. No other places are allowed to have their own references to it,
/// strong or weak. The reason is that in the future the engine will support dynamic reconfiguration
/// of graphical settings, which will require the renderer to be capable of a full restart at any
/// point.
pub struct DeviceContext {
    pub device: Device,
    pub queue: Queue,
    pub instance: Instance,
    pub adapter: Adapter,
}

/// Various feature flags of the renderer, determined at runtime from the engine's settings and the
/// user's hardware configuration.
pub struct RenderCapabilities {
    /// Specifies whether BC/DXT compression is allowed.
    pub allow_bc_compression: bool,
}

/// Combined renderer state, actually owning various wgpu resources.
///
/// ## Access
/// Access to the render state is split between the scene thread and the render thread:
///  * scene thread accesses it during main processing
///  * render thread accesses it during post processing
pub struct Renderer {
    dc: Arc<DeviceContext>,
    pub capabilities: RenderCapabilities,

    pub egui_output: Arc<Mutex<EguiOutput>>,

    // If you add anything new, don't forget to collect_garbage() it
    pub cameras: ArcPool<CameraResource>,
    pub cubemaps: ArcPool<CubemapResource>,
    pub textures: ArcPool<TextureResource>,
    pub skyboxes: ArcPool<SkyboxResource>,
}

/// Common & miscellaneous functions
impl Renderer {
    pub fn new(dc: Arc<DeviceContext>, egui_output: Arc<Mutex<EguiOutput>>) -> Self {
        Self {
            capabilities: RenderCapabilities {
                allow_bc_compression: dc
                    .adapter
                    .features()
                    .contains(wgpu::Features::TEXTURE_COMPRESSION_BC),
            },

            dc,
            egui_output,

            cameras: ArcPool::with_growth_size(10),
            cubemaps: ArcPool::with_growth_size(10),
            textures: ArcPool::with_growth_size(10),
            skyboxes: ArcPool::with_growth_size(10),
        }
    }

    pub fn dc(&self) -> &DeviceContext {
        &self.dc
    }

    pub fn collect_garbage(&mut self) {
        self.cameras.collect_garbage();
        self.cubemaps.collect_garbage();
        self.textures.collect_garbage();
        self.skyboxes.collect_garbage();
    }
}

// boilerplate generator 666
// TODO: simplify this macro (would likely need some crate for macro_rules writing to aid with this)
macro_rules! resource_functions {
    ($({ $name:ident $field:ident : $Type:ty, $Handle:ty, $Descriptor:ty })*) => {
        paste! {
            $(
                impl Renderer {
                    pub fn [< create_ $name >](&mut self, desc: &$Descriptor) -> $Handle {
                        $Handle(self.$field.allocate(
                            <$Type>::new(&self.dc, desc)
                        ))
                    }

                    pub fn [< get_ $name >](&self, handle: &$Handle) -> &$Type {
                        self.$field.get(&handle.0)
                    }

                    pub fn [< get_ $name _mut >](&mut self, handle: &$Handle) -> &mut $Type {
                        self.$field.get_mut(&handle.0)
                    }
                }
            )*
        }
    };
}

resource_functions! {
    { camera  cameras  : CameraResource,  CameraHandle,  CameraDescriptor  }
    { texture textures : TextureResource, TextureHandle, TextureDescriptor }
    { cubemap cubemaps : CubemapResource, CubemapHandle, CubemapDescriptor }
    { skybox  skyboxes : SkyboxResource,  SkyboxHandle,  SkyboxDescriptor  }
}
