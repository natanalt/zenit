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

use self::imgui_renderer::ImguiFrame;
use ahash::AHashMap;
use log::*;
use paste::paste;
use pollster::FutureExt;
use std::sync::Arc;
use wgpu::{Features, Limits, TextureFormat};
use winit::window::Window;
use zenit_utils::ArcPool;

mod frame_state;
pub mod imgui_renderer;
pub mod macros;
pub mod system;

#[doc(inline)]
pub use resources::*;
mod resources;

#[doc(inline)]
pub use scene::*;

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
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
}

impl DeviceContext {
    /// Creates a new [`wgpu`] instance and initializes a whole device context based from that.
    pub fn create(window: &Window) -> (Arc<Self>, wgpu::Surface, wgpu::SurfaceConfiguration) {
        info!("Creating a device context...");

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            ..Default::default()
        });

        let surface = unsafe {
            instance
                .create_surface(&window)
                .expect("couldn't create the surface")
        };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .block_on()
            .expect("couldn't find a GPU");

        info!("Using adapter: {}", adapter.get_info().name);
        info!("Using backend: {:?}", adapter.get_info().backend);

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    // BC compression, aka DXTn or S3
                    features: Features::TEXTURE_COMPRESSION_BC | Features::CLEAR_TEXTURE,
                    limits: Limits::default(),
                },
                None,
            )
            .block_on()
            .expect("couldn't initialize the device");

        // We could establish better error handling here
        device.on_uncaptured_error(Box::new(|error| {
            error!("An error has been reported by wgpu!");
            error!("{error}");
            panic!("Graphics API error: {error}");
        }));

        let sconfig = surface
            .get_default_config(
                &adapter,
                window.inner_size().width,
                window.inner_size().height,
            )
            .expect("surface unsupported by adapter");

        surface.configure(&device, &sconfig);

        let result = Arc::new(Self {
            device,
            queue,
            instance,
            adapter,
        });

        (result, surface, sconfig)
    }
}

/// Various feature flags of the renderer, determined at runtime from the engine's settings and the
/// user's hardware configuration.
pub struct RenderCapabilities {
    /// Specifies whether BC/DXT compression is allowed.
    pub allow_bc_compression: bool,
}

/// Public interface to the renderer API. It owns various renderer resources stored within dedicated
/// pools.
///
/// ## Access (synchronization internals)
/// Access to the public renderer is split between the scene thread and the render thread:
///  * scene thread accesses it during main processing
///  * render thread accesses it during post processing
pub struct Renderer {
    dc: Arc<DeviceContext>,

    pub capabilities: RenderCapabilities,

    pub cameras: Cameras,
    pub cubemaps: Cubemaps,
    pub textures: Textures,
    pub skyboxes: Skyboxes,

    /// Map of named shaders.
    ///
    /// You may be wondering why these aren't getting a dedicated pool, like all other GPU resources.
    /// Well, there's just no real reason to use a whole pool, string identifiers are good enough:tm:
    pub shaders: AHashMap<String, ShaderResource>,

    /// Currently rendered frame for Dear ImGui.
    ///
    /// The render system accesses it during the post-frame stage, as part of frame state collection.
    /// The scene thread can use it all it wants during main processing.
    pub imgui_frame: Option<ImguiFrame>,

    // Miscellaneous stuff for functions
    filtered_sampler: Arc<wgpu::Sampler>,
    unfiltered_sampler: Arc<wgpu::Sampler>,
}

/// Common & miscellaneous functions
impl Renderer {
    pub fn new(
        window: &Window,
    ) -> (
        Self,
        Arc<DeviceContext>,
        wgpu::Surface,
        wgpu::SurfaceConfiguration,
    ) {
        let (dc, surface, sconfig) = DeviceContext::create(window);
        let render_system_dc = dc.clone();

        (
            Self {
                capabilities: RenderCapabilities {
                    allow_bc_compression: dc
                        .adapter
                        .features()
                        .contains(wgpu::Features::TEXTURE_COMPRESSION_BC),
                },

                cameras: Cameras::with_growth_size(10),
                cubemaps: Cubemaps::with_growth_size(10),
                textures: Textures::with_growth_size(10),
                skyboxes: Skyboxes::with_growth_size(10),
                shaders: AHashMap::with_capacity(10),

                imgui_frame: None,

                unfiltered_sampler: Arc::new(dc.device.create_sampler(&wgpu::SamplerDescriptor {
                    label: Some("unfiltered sampler"),
                    address_mode_u: wgpu::AddressMode::Repeat,
                    address_mode_v: wgpu::AddressMode::Repeat,
                    address_mode_w: wgpu::AddressMode::Repeat,
                    mag_filter: wgpu::FilterMode::Nearest,
                    min_filter: wgpu::FilterMode::Nearest,
                    mipmap_filter: wgpu::FilterMode::Nearest,
                    lod_min_clamp: 0.0,
                    lod_max_clamp: 32.0,
                    compare: None,
                    anisotropy_clamp: 1,
                    border_color: None,
                })),

                filtered_sampler: Arc::new(dc.device.create_sampler(&wgpu::SamplerDescriptor {
                    label: Some("filtered sampler"),
                    address_mode_u: wgpu::AddressMode::Repeat,
                    address_mode_v: wgpu::AddressMode::Repeat,
                    address_mode_w: wgpu::AddressMode::Repeat,
                    mag_filter: wgpu::FilterMode::Linear,
                    min_filter: wgpu::FilterMode::Linear,
                    mipmap_filter: wgpu::FilterMode::Linear,
                    lod_min_clamp: 0.0,
                    lod_max_clamp: 32.0,
                    compare: None,
                    anisotropy_clamp: 1,
                    border_color: None,
                })),

                dc,
            },
            render_system_dc,
            surface,
            sconfig,
        )
    }

    pub fn dc(&self) -> &DeviceContext {
        &self.dc
    }
}

// boilerplate generator 666
macro_rules! resources {
    ($({ $Name:ident $field:ident $singular:ident : $Resource:ty, $Handle:ty, $Descriptor:ty })*) => {
        paste! {
            $(
                //#[doc = stringify!("Container of [`", $Resource, "`] resources with a typed [`", $Handle, "`] handle access")]
                pub struct $Name {
                    pub raw_pool: ArcPool<$Resource>,
                }

                impl $Name {
                    fn with_growth_size(size: u32) -> Self {
                        Self {
                            raw_pool: ArcPool::with_growth_size(size),
                        }
                    }

                    pub fn get(&self, handle: &$Handle) -> &$Resource {
                        self.raw_pool.get(&handle.0)
                    }

                    pub fn get_mut(&mut self, handle: &$Handle) -> &mut $Resource {
                        self.raw_pool.get_mut(&handle.0)
                    }

                    pub fn collect_garbage(&mut self) {
                        self.raw_pool.collect_garbage();
                    }
                }

                impl Renderer {
                    pub fn [<create_ $singular>](&mut self, desc: &$Descriptor) -> $Handle {
                        let resource = <$Resource>::new(self, desc);
                        $Handle(self.$field.raw_pool.allocate(resource))
                    }
                }
            )*

            impl Renderer {
                pub fn collect_garbage(&mut self) {
                    $(
                        self.$field.collect_garbage();
                    )*
                }
            }
        }
    };
}

resources! {
    { Cameras  cameras  camera  : CameraResource,  CameraHandle,  CameraDescriptor  }
    { Textures textures texture : TextureResource, TextureHandle, TextureDescriptor }
    { Cubemaps cubemaps cubemap : CubemapResource, CubemapHandle, CubemapDescriptor }
    { Skyboxes skyboxes skybox  : SkyboxResource,  SkyboxHandle,  SkyboxDescriptor  }
}
