use super::{
    imgui_renderer::{ImGuiRenderData, ImGuiTexture},
    system::GraphicsSystem,
    BuiltScene, CameraDescriptor, CameraHandle, CameraResource, CubemapDescriptor, CubemapHandle,
    CubemapResource, DeviceContext, RenderCapabilities, ShaderResource, SkyboxDescriptor,
    SkyboxHandle, SkyboxResource, TextureDescriptor, TextureHandle, TextureResource,
};
use crate::assets::shader_loader::load_builtin_shaders;
use ahash::AHashMap;
use imgui::TextureId;
use std::sync::Arc;
use winit::window::Window;
use zenit_utils::ArcPool;

/// Public interface to the renderer API. It owns various renderer resources stored within dedicated
/// pools.
///
/// ## Access (synchronization internals)
/// Access to the public renderer is split between the scene thread and the render thread:
///  * scene thread accesses it during main processing
///  * render thread accesses it during post processing
pub struct Renderer {
    /// **Do not clone this Arc**.
    /// It's provided for convenience as a pointer to the [`DeviceContext`].
    pub dc: Arc<DeviceContext>,

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

    pub(in crate::graphics) pending_frame: PendingFrame,
    top_imgui_texture_id: usize,

    // Miscellaneous stuff for graphics functions
    pub(in crate::graphics) filtered_sampler: Arc<wgpu::Sampler>,
    pub(in crate::graphics) unfiltered_sampler: Arc<wgpu::Sampler>,
    pub(in crate::graphics) shared_bind_layouts: AHashMap<String, wgpu::BindGroupLayout>,
}

/// Common & miscellaneous functions
impl Renderer {
    pub fn new(window: &Arc<Window>) -> (Self, GraphicsSystem) {
        let (dc, surface, sconfig) = DeviceContext::create(window);

        let mut renderer = Self {
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

            pending_frame: PendingFrame::default(),
            top_imgui_texture_id: 1, // 0 is a reserved constant for the font atlas

            shared_bind_layouts: AHashMap::with_capacity(10),

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

            dc: dc.clone(),
        };

        load_builtin_shaders(&mut renderer).expect("couldn't load a shader");

        let graphics_system =
            GraphicsSystem::new(&mut renderer, dc.clone(), window.clone(), surface, sconfig);

        (renderer, graphics_system)
    }

    pub fn submit_imgui(&mut self, render_data: ImGuiRenderData) {
        debug_assert!(self.pending_frame.imgui_render_data.is_none());
        self.pending_frame.imgui_render_data = Some(render_data);
    }
}

/// Dear ImGui functions
impl Renderer {
    pub fn add_imgui_texture(&mut self, texture: TextureHandle) -> Arc<TextureId> {
        let id = Arc::new(self.next_imgui_texture_id());
        self.pending_frame
            .imgui_new_textures
            .push((id.clone(), ImGuiTexture::from_texture(texture)));
        id
    }

    pub fn add_imgui_camera(&mut self, camera: CameraHandle) -> Arc<TextureId> {
        let id = Arc::new(self.next_imgui_texture_id());
        self.pending_frame
            .imgui_new_textures
            .push((id.clone(), ImGuiTexture::from_camera(camera)));
        id
    }

    pub fn update_imgui_font(&mut self, font: TextureHandle) {
        self.pending_frame.imgui_new_font = Some(ImGuiTexture::from_texture(font));
    }

    fn next_imgui_texture_id(&mut self) -> TextureId {
        let id = self.top_imgui_texture_id;
        self.top_imgui_texture_id = id
            .checked_add(1)
            .expect("somehow the texture ID accumulator overflowed");
        imgui::TextureId::new(id)
    }
}

#[derive(Default)]
pub(in crate::graphics) struct PendingFrame {
    pub imgui_render_data: Option<ImGuiRenderData>,
    pub imgui_new_textures: Vec<(Arc<TextureId>, ImGuiTexture)>,
    pub imgui_new_font: Option<ImGuiTexture>,

    pub scenes: Vec<BuiltScene>,
}

// boilerplate generator 666
macro_rules! resources {
    ($({ $Name:ident $field:ident $singular:ident : $Resource:ty, $Handle:ty, $Descriptor:ty })*) => {
        ::paste::paste! {
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

                    pub fn iter(&self) -> impl Iterator<Item = &$Resource> {
                        self.raw_pool.iter()
                    }

                    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut $Resource> {
                        self.raw_pool.iter_mut()
                    }

                    pub fn iter_handles(&self) -> impl Iterator<Item = (&$Resource, $Handle)> {
                        self.raw_pool
                            .iter_handles()
                            .map(|(value, handle)| (value, $Handle(handle)))
                    }

                    pub fn count_elements(&self) -> usize {
                        self.iter().count()
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
