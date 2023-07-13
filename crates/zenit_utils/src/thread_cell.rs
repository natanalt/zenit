use std::{
    cell::UnsafeCell,
    thread::{self, ThreadId},
};

// TODO: verify if ThreadCell is is fully sound

/// The thread cell stores types that aren't [`Send`], while itself being [`Send`] by ensuring that
/// the underlying value can only be accessed on the creator thread.
/// 
/// This structures stores the thread ID of the object's owner within itself, guaranteeing that every
/// access happens on the owner thread.
///
/// ## Important drop notes
/// The [`Drop`] implementation for the stored value is *not* called automatically. It can only be done
/// by consuming the inner value by the owner thread.
///
/// Don't forget.
pub struct ThreadCell<T> {
    owner: ThreadId,
    cell: UnsafeCell<T>,
}

impl<T> ThreadCell<T> {
    /// Creates a new [`ThreadCell`] with a selected value.
    pub fn new(value: T) -> Self {
        Self {
            owner: thread::current().id(),
            cell: UnsafeCell::new(value),
        }
    }

    /// Returns an immutable reference to the underlying value.
    /// 
    /// ## Panics
    /// Panics if the function isn't called from the owner thread.
    pub fn get(&self) -> &T {
        self.ensure_owner_thread();
        unsafe { self.cell.get().as_ref().unwrap() }
    }

    /// Returns a mutable reference to the underlying value.
    /// 
    /// ## Panics
    /// Panics if the function isn't called from the owner thread.
    pub fn get_mut(&mut self) -> &mut T {
        self.ensure_owner_thread();
        unsafe { self.cell.get().as_mut().unwrap() }
    }

    /// Consumes the cell, returning the underlying value.
    /// 
    /// ## Panics
    /// Panics if the function isn't called from the owner thread.
    pub fn take(self) -> T {
        self.ensure_owner_thread();
        self.cell.into_inner()
    }

    fn ensure_owner_thread(&self) {
        if thread::current().id() != self.owner {
            panic!("thread cell access from an invalid thread");
        }
    }
}

// Safety: thread is checked during accesses
unsafe impl<T> Send for ThreadCell<T> {}
