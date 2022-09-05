use std::sync::Arc;
use crate::engine::system::HasSystemInterface;
use super::{RenderSystem, RenderCommunication, Viewport};

/// External system interface for interacting with the renderer
pub struct RenderInterface(pub(super) Arc<RenderCommunication>);

impl RenderInterface {
    pub fn create_viewport(&self) -> Arc<Viewport> {
        todo!()
    }

    pub fn create_scenario(&self) {
        todo!()
    }
}
