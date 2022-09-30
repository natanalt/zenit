use std::{
    cell::UnsafeCell,
    mem,
    thread::{self, ThreadId},
};

/// The thread cell is a solution for types that need to stay `Send + Sync`,
/// while holding singlethreaded data.
///
/// This structure associates itself with the creator's thread ID, and only
/// allows taking its references after a safety check to prove that it's
/// still the creator therad using it.
///
/// ## Important drop notes
/// The [`Drop`] implementation for the stored value is *not* called
/// automatically. It can only be done by consuming the inner value by the
/// owner thread.
///
/// If the cell value is not taken out, it will be forgotten, which can have
/// many nasty consequences, depending on what kind of value you're storing
/// here.
///
/// Don't forget.
pub struct ThreadCell<T> {
    owner: ThreadId,
    cell: Option<UnsafeCell<T>>,
}

impl<T> ThreadCell<T> {
    /// Returns a thread cell that cannot be ever retrieved
    ///
    /// Don't do this too much lol
    pub fn invalid() -> Self {
        // Spawn a blank thread to generate a new ID
        let tid = thread::spawn(move || {}).thread().id();

        Self {
            owner: tid,
            cell: None,
        }
    }

    pub fn new(value: T) -> Self {
        Self {
            owner: thread::current().id(),
            cell: Some(UnsafeCell::new(value)),
        }
    }

    pub fn get(&self) -> Option<&T> {
        if self.owner == thread::current().id() {
            // Safety: thread was verified
            unsafe { Some(self.cell.as_ref()?.get().as_ref().unwrap()) }
        } else {
            None
        }
    }

    pub fn get_mut(&mut self) -> Option<&mut T> {
        if self.owner == thread::current().id() {
            // Safety: thread was verified
            unsafe { Some(self.cell.as_ref()?.get().as_mut().unwrap()) }
        } else {
            None
        }
    }

    pub fn take(&mut self) -> Option<T> {
        if self.owner == thread::current().id() {
            Some(self.cell.take()?.into_inner())
        } else {
            None
        }
    }

    pub fn clear(&mut self) -> Result<(), ()> {
        if self.owner == thread::current().id() {
            let _ = self.take();
            Ok(())
        } else {
            Err(())
        }
    }
}

impl<T> Drop for ThreadCell<T> {
    fn drop(&mut self) {
        // See struct docs for details
        if let Some(value) = self.cell.take() {
            mem::forget(value);
        }
    }
}

// Safety: thread is checked during accesses
unsafe impl<T> Send for ThreadCell<T> {}
unsafe impl<T> Sync for ThreadCell<T> {}
