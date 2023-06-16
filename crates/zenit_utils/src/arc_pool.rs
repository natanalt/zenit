use std::{
    hash::{Hash, Hasher},
    mem,
    num::NonZeroU32,
    sync::{Arc, Weak},
};

/// Handle to an element in an arc pool
#[derive(Debug, Clone)]
pub struct ArcPoolHandle(Arc<u32>);

impl PartialEq for ArcPoolHandle {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for ArcPoolHandle {}

impl Hash for ArcPoolHandle {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.0).hash(state);
    }
}

/// Atomically reference counted generic pool.
///
/// Similarly to the generic [`crate::Pool`], this allows to store a set of `T` elements, and
/// access them via specific handles. The difference is that handles in `ArcPool` are backed
/// internally by an [`Arc`]. Once the reference counted value dies, its slot can be reclaimed
/// (which is done automatically when needed during allocation, or manually via
/// [`Self::collect_garbage`]).
pub struct ArcPool<T> {
    free_indices: Vec<u32>,
    values: Vec<Option<(Weak<u32>, T)>>,
    pub growth_size: NonZeroU32,
}

impl<T> ArcPool<T> {
    pub fn with_growth_size(size: u32) -> Self {
        Self {
            free_indices: vec![],
            values: vec![],
            growth_size: NonZeroU32::new(size).expect("growth size can't be zero"),
        }
    }

    pub fn allocate(&mut self, initial: T) -> ArcPoolHandle {
        let index = match self.free_indices.pop() {
            Some(index) => index,
            None => {
                if self.collect_garbage() == 0 {
                    // Out of indices, grow all vecs
                    let low_index = self.values.len() as u32;
                    let high_index = low_index
                        .checked_add(self.growth_size.get())
                        .expect("pool index overflow");
                    let growth_range = low_index..high_index;

                    // Reverse index range, so that pop gets lowest entries
                    self.free_indices.extend(growth_range.clone().rev());
                    self.values.extend(growth_range.map(|_| None));
                }
                self.free_indices.pop().unwrap()
            }
        };

        let handle = Arc::new(index);
        self.values[index as usize] = Some((Arc::downgrade(&handle), initial));
        ArcPoolHandle(handle)
    }

    /// Returns an immutable reference to an element through its handle.
    ///
    /// ## Panics
    /// Panics if the handle is invalid (points to an invalid index or a dead element, which
    /// may happen when handles from different pools are mixed up)
    pub fn get(&self, handle: &ArcPoolHandle) -> &T {
        &self
            .values
            .get(*handle.0 as usize)
            .expect("invalid index in a live arc pool reference")
            .as_ref()
            .expect("invalid dead value in a live arc pool reference")
            .1
    }

    /// Returns a mutable reference to an element through its handle.
    ///
    /// ## Panics
    /// Panics if the handle is invalid (points to an invalid index or a dead element, which
    /// may happen when handles from different pools are mixed up)
    pub fn get_mut(&mut self, handle: &ArcPoolHandle) -> &mut T {
        &mut self
            .values
            .get_mut(*handle.0 as usize)
            .expect("invalid index in a live arc pool reference")
            .as_mut()
            .expect("invalid dead value in a live arc pool reference")
            .1
    }

    pub fn set(&mut self, handle: &ArcPoolHandle, value: T) -> T {
        mem::replace(self.get_mut(handle), value)
    }

    /// Drops any values without live references, freeing up their indices.
    ///
    /// Returns the amount of freed entries.
    pub fn collect_garbage(&mut self) -> u32 {
        let mut freed = 0;
        for (index, handle) in self.values.iter_mut().enumerate() {
            if let Some(tuple) = handle.as_mut() {
                if tuple.0.strong_count() == 0 {
                    *handle = None;
                    self.free_indices.push(index as u32);
                    freed += 1;
                }
            }
        }
        freed
    }
}
