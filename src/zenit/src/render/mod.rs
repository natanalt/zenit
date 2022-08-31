//! Zenit Render System
//! 
//! ## Usage
//! Generally, the scene system exposes a lot of common rendering functionality,
//! to be used by usual game code.
//! 
//! Any rendering
//! 
//! ### Frame lifetime
//! 

use std::sync::{Arc, Barrier};
use crate::engine::system::{System, HasSystemInterface, SystemContext};

mod interface;
pub use interface::RenderInterface;
mod viewport;
pub use viewport::Viewport;
pub use viewport::ViewportCreationInfo;

/// The render system itself. See module documentation for detaisl
pub struct RenderSystem {
    comms: Arc<RenderCommunication>,
}

impl RenderSystem {
    pub fn new() -> Self {
        Self {
            comms: Arc::new(RenderCommunication {  }),
        }
    }
}

impl<'ctx> System<'ctx> for RenderSystem {
    fn name(&self) -> &str {
        "Render System"
    }

    fn frame(&mut self, context: &mut SystemContext<'ctx>) {

        context.finish_frame_phase();

        todo!()
    }
}

impl HasSystemInterface for RenderSystem {
    type SystemInterface = RenderInterface;

    fn create_system_interface(&self) -> RenderInterface {
        debug_assert!(Arc::strong_count(&self.comms) <= 2, "comms duplicated?");

        RenderInterface {
            comms: self.comms.clone(),
        }
    }
}

/// Communication between [`RenderSystem`] and [`RenderInterface`].
struct RenderCommunication {
}
