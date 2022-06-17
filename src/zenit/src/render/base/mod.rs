pub mod shaders;
pub mod surface;
pub mod texture;
pub mod utils;
pub mod mipmaps;

/// Shared structure containing various wgpu drawing objects
pub struct RenderContext {
    pub device: wgpu::Device,
    pub adapter: wgpu::Adapter,
    pub adapter_info: wgpu::AdapterInfo,
    pub queue: wgpu::Queue,
}
