//! Engine-global data accessible by all systems

use std::{any::Any, sync::Arc};

/// Represents a piece of globally readable data
pub trait Data: Any + Send + Sync {
    /// What kind of data is stored by this container?
    type Storage: Sized;

    /// Reads the current value of this data
    fn read(&self) -> Self::Storage;
}

impl<T: Send + Sync + 'static> Data for Arc<T> {
    type Storage = Arc<T>;

    fn read(&self) -> Self::Storage {
        self.clone()
    }
}
