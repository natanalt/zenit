use std::cell::RefCell;

/// Zenit-specific extensions for [`RefCell`].
pub trait RefCellExt<T> {
    /// Calls the closure for the given mutable reference to T.
    /// 
    /// ## Panics
    /// Panics if the [`RefCell`] is already borrowed.
    fn with<R>(&self, f: impl FnOnce(&mut T) -> T) -> T;
}

impl<T> RefCellExt<T> for RefCell<T> {
    fn with<R>(&self, f: impl FnOnce(&mut T) -> T) -> T {
        let mut borrow = self.borrow_mut();
        f(&mut borrow)
    }
}
