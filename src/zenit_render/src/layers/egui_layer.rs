use crate::base::{screen::RenderLayer, target::RenderTarget, context::RenderContext};
use std::sync::{Arc, RwLock};

/// A layer for integrating egui
/// 
/// The integration takes an `Arc<RwLock<...>>` handle to [`egui::Context`].
/// When the layer is rendered, it's read-locked, tesellated and rendered.
/// 
/// This requires the context to be processed *before* it's rendered, which
/// should happen anyway given how Zenit is organized. 
pub struct EguiLayer {
    pub context: Arc<RwLock<egui::Context>>,
}

impl EguiLayer {
    pub fn new() -> Self {
        todo!()
    }
}

impl RenderLayer for EguiLayer {
    fn render(
        &self,
        context: &Arc<RenderContext>,
        target: &dyn RenderTarget,
    ) -> Vec<wgpu::CommandBuffer> {
        let context = self.context.read().unwrap();


        todo!()
    }
}
