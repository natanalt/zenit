use super::{
    resources::{Shader, ShaderManager},
    DeviceContext, system::RenderSystem,
};
use std::sync::Arc;
use wgpu::*;
use zenit_utils::AnyResult;

/// The renderer is the public interface for anything related to the renderer.
///
/// A mutex to it is available in the global context. Keep in mind, that it's a mutex protected
/// object, primarily used by the scene thread during the main processing.
pub struct Renderer {
    dc: Arc<DeviceContext>,

    shader_manager: ShaderManager,
}

impl Renderer {
    pub fn new(rs: &mut RenderSystem) -> Self {
        let dc = rs.dc.clone();

        let shader_manager = ShaderManager::new();

        Self {
            dc,
            shader_manager,
        }
    }

    /// Loads and compiles a shader. See the [`Shader`] documentation for details.
    /// 
    /// The shaders are cached.
    pub fn load_shader(&mut self, path: &str) -> AnyResult<Arc<Shader>> {
        self.shader_manager.load_shader(&self.dc, path)
    }

    /// Cleans up any unused resources, like any caches.
    pub fn cleanup(&mut self) {
        self.shader_manager.cleanup();
    }
}
