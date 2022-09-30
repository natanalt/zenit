use super::{RenderCommunication, Viewport};
use std::sync::Arc;

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
