use super::{context::RenderContext, target::RenderTarget};
use std::{borrow::Cow, sync::Arc};

pub struct Screen {
    pub label: Option<Cow<'static, str>>,
    pub target: Arc<dyn RenderTarget>,
    pub layers: Vec<Arc<dyn RenderLayer>>,
}

pub trait RenderLayer {
    fn render(
        &self,
        context: &Arc<RenderContext>,
        target: &dyn RenderTarget,
    ) -> Vec<wgpu::CommandBuffer>;
}
