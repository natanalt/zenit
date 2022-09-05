use std::{
    cell::UnsafeCell,
    thread::{self, ThreadId},
};

/// The thread cell is a solution for types that need to say `Send + Sync`,
/// while holding singlethreaded data.
///
/// This structure associates itself with the creator's thread ID, and only
/// allows taking its references after a safety check to prove that it's
/// still the creator therad using it.
pub struct ThreadCell<T> {
    owner: ThreadId,
    cell: UnsafeCell<T>,
}

impl<T> ThreadCell<T> {
    pub fn new(value: T) -> Self {
        Self {
            owner: thread::current().id(),
            cell: UnsafeCell::new(value),
        }
    }

    pub fn get(&self) -> Option<&T> {
        self.owner.eq(&thread::current().id()).then(|| {
            // Safety: thread was verified
            unsafe { self.cell.get().as_ref().unwrap() }
        })
    }

    pub fn get_mut(&mut self) -> Option<&mut T> {
        self.owner.eq(&thread::current().id()).then(|| {
            // Safety: thread was verified
            unsafe { self.cell.get().as_mut().unwrap() }
        })
    }
}

// Safety: thread is checked during accesses
unsafe impl<T> Send for ThreadCell<T> {}
unsafe impl<T> Sync for ThreadCell<T> {}
