use super::{context::RenderContext, target::RenderTarget};
use std::{borrow::Cow, sync::Arc};

pub struct Screen {
    pub label: Option<Cow<'static, str>>,
    pub target: Arc<dyn RenderTarget>,
    pub layers: Vec<Box<dyn RenderLayer>>,
}

pub trait RenderLayer: Send + Sync {
    fn render(
        &self,
        context: &Arc<RenderContext>,
        target: &dyn RenderTarget,
    ) -> Vec<wgpu::CommandBuffer>;
}
