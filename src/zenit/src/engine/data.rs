//! Engine-global data accessible by all systems

use std::any::Any;

/// Represents a piece of globally readable data
pub trait Data: Any + Send + Sync {
    type Storage: Sized;
    fn get_data(&self) -> &Self::Storage;
}
