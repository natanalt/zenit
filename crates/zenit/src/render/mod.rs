use wgpu::*;

pub mod macros;
pub mod resources;
pub mod system;
pub mod api;
pub mod components;
mod frame_state;

// TODO: make the RENDER_FORMAT dynamically chosen?

/// Format commonly used for render textures. This is the format to be used by render pipelines
/// and anything else that depends on a constant TextureFormat.
/// 
/// Currently it defaults to a 16-bit float format to enable HDR support.
pub const RENDER_FORMAT: TextureFormat = TextureFormat::Rgba16Float;

/// The device context contains public information regarding the current [`wgpu`] instance,
/// including the device, queue, adapter, main surface, etc.
/// 
/// An Arc to it is available in the global context.
pub struct DeviceContext {
    pub device: Device,
    pub queue: Queue,
    pub instance: Instance,
    pub adapter: Adapter,
}
