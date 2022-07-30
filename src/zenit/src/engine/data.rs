//! Engine-global data accessible by all systems

use std::{any::Any, sync::Arc};

/// Represents a piece of globally readable data
pub trait Data: Any + Send + Sync {
    type Storage: Sized;
    fn get_data(&self) -> &Self::Storage;
}

impl<T> Data for Arc<T>
where
    Self: Any + Send + Sync,
    T: Send + Sync + Sized,
{
    type Storage = T;

    fn get_data(&self) -> &Self::Storage {
        self
    }
}
