use std::sync::Arc;
use glam::IVec2;

pub trait RenderTarget {
    /// Returns the current [`wgpu::TextureView`] that can be used as a valid
    /// render target.
    /// 
    /// Value returned is enclosed within an [`Arc`], that can be assumed to be
    /// only usable for the duration of the current frame.
    fn get_current_view(&self) -> Arc<wgpu::TextureView>;
    fn get_size(&self) -> IVec2;
}
