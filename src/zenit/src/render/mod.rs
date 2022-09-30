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

use crate::engine::system::{HasSystemInterface, System, SystemContext};
use std::sync::Arc;

mod interface;
pub use interface::RenderInterface;

mod viewport;
pub use viewport::Viewport;
pub use viewport::ViewportCreationInfo;

mod scenario;
pub use scenario::Scenario;

/// The render system itself. See module documentation for detaisl
pub struct RenderSystem {
    comms: Arc<RenderCommunication>,
}

impl Default for RenderSystem {
    fn default() -> Self {
        Self {
            comms: Arc::new(RenderCommunication {}),
        }
    }
}

impl<'ctx> System<'ctx> for RenderSystem {
    fn name(&self) -> &str {
        "Render System"
    }

    fn frame<'a>(&mut self, context: &mut SystemContext<'ctx, 'a>) {
        let _ = context;
    }
}

impl HasSystemInterface for RenderSystem {
    type SystemInterface = RenderInterface;

    fn create_system_interface(&self) -> RenderInterface {
        debug_assert!(Arc::strong_count(&self.comms) <= 2, "comms duplicated?");
        RenderInterface(self.comms.clone())
    }
}

/// Communication between [`RenderSystem`] and [`RenderInterface`].
struct RenderCommunication {}
