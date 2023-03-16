use std::sync::Arc;
use glam::*;
use wgpu::*;
use zenit_utils::{math::{Radians, AngleExt}, ArcPool};

pub mod frame_state;
pub mod macros;
pub mod system;

mod texture;
pub use texture::*;
mod camera;
pub use camera::*;
mod scene;
pub use scene::*;

// TODO: make the RENDER_FORMAT dynamically chosen?

/// Format commonly used for render textures. This is the format to be used by render pipelines
/// and anything else that depends on a constant TextureFormat.
///
/// Currently it defaults to a 16-bit float format to enable HDR support.
pub const RENDER_FORMAT: TextureFormat = TextureFormat::Rgba16Float;

/// The device context contains public information regarding the current [`wgpu`] instance,
/// including the device, queue, adapter, main surface, etc.
pub struct DeviceContext {
    pub device: Device,
    pub queue: Queue,
    pub instance: Instance,
    pub adapter: Adapter,
}

/// Combined renderer state, actually owning various wgpu resources.
///
/// ## Access
/// Access to the render state is split between the scene thread and the render thread:
///  * scene thread accesses it during main processing
///  * render thread accesses it during post processing
pub struct Renderer {
    pub dc: Arc<DeviceContext>,
    pub cameras: ArcPool<CameraResource>,
    pub textures: ArcPool<TextureResource>,
}

impl Renderer {
    pub fn new(dc: Arc<DeviceContext>) -> Self {
        Self {
            dc,
            cameras: ArcPool::with_growth_size(10),
            textures: ArcPool::with_growth_size(10),
        }
    }
}

/// Texture functions
impl Renderer {
    pub fn create_2d_texture(&mut self, desc: &TextureDescriptor2D) -> TextureHandle {
        todo!()
    }
}

pub struct TextureDescriptor2D {
    pub name: String,
    pub size: UVec2,
    pub mip_levels: u32,
}

/// Camera functions
impl Renderer {
    pub fn create_camera(&mut self, desc: &CameraDescriptor) -> CameraHandle {
        todo!()
    }

    pub fn get_camera(&self, handle: &CameraHandle) -> &CameraResource {
        self.cameras.get(&handle.0)
    }

    pub fn get_camera_mut(&mut self, handle: &CameraHandle) -> &mut CameraResource {
        self.cameras.get_mut(&handle.0)
    }
}

pub struct CameraDescriptor {
    pub resolution: UVec2,
    pub fov: Radians,
    pub near_plane: f32,
    pub far_plane: f32,
}

impl Default for CameraDescriptor {
    fn default() -> Self {
        Self {
            resolution: uvec2(640, 480),
            fov: 90f32.radians(),
            near_plane: 0.0001,
            far_plane: 1000.0,
        }
    }
}
