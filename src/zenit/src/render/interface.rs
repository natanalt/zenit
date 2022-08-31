use std::sync::Arc;
use crate::engine::system::HasSystemInterface;
use super::{RenderSystem, RenderCommunication};

/// External system interface for interacting with the renderer
pub struct RenderInterface {
    pub(super) comms: Arc<RenderCommunication>,
}

impl RenderInterface {
    pub fn create_viewport(&self) {
        todo!()
    }
}
