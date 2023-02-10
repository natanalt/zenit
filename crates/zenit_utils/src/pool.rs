//! Generational object pools

use std::{
    cell::{Ref, RefCell, RefMut},
    convert::identity,
    iter,
};

type PoolEntry<T> = Option<(RefCell<T>, u64)>;

/// A generational, single-threaded object pool with individual object locks backed by [`RefCell`].
#[derive(Clone)]
pub struct Pool<T> {
    // TODO: store free index lists you dumb and don't search the list all the time
    list: Vec<PoolEntry<T>>,
    pub growth_size: usize,
    generation: u64,
    lowest_free: usize,
}

impl<T> Pool<T> {
    pub fn new() -> Self {
        // just a wild guess
        Self::with_capacity(8)
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            list: iter::from_fn(|| None).take(cap).collect(),
            growth_size: 8,
            generation: 1,
            lowest_free: 0,
        }
    }

    pub fn allocate(&mut self, initial: T) -> PoolDescriptor {
        let generation = self.next_gen();
        let (index, entry) = self.next_empty();
        *entry = Some((RefCell::new(initial), generation));
        PoolDescriptor { index, generation }
    }

    pub fn free(&mut self, desc: PoolDescriptor) {
        assert!(self.is_valid(desc));

        if self.lowest_free > desc.index {
            self.lowest_free = desc.index;
        }

        self.list[desc.index] = None;
    }

    /// Immutably iterates over the pool, returning [`RefCell`] values of present objects.
    pub fn iter(&self) -> impl Iterator<Item = &RefCell<T>> {
        self.list
            .iter()
            .map(Option::as_ref)
            .filter_map(identity)
            .map(|(cell, _)| cell)
    }

    /// Mutably iterates over the pool, returning mutable references to present objects.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.list
            .iter_mut()
            .map(Option::as_mut)
            .filter_map(identity)
            .map(|(cell, _)| cell.get_mut())
    }

    /// Retains only the elements of the pool for which the closure returns true.
    pub fn retain(&mut self, mut f: impl FnMut(&mut T) -> bool) {
        for (index, option) in self.list.iter_mut().enumerate() {
            let Some((cell, _)) = option else { continue };
            if !f(cell.get_mut()) {
                *option = None;
                if self.lowest_free > index {
                    self.lowest_free = index;
                }
            }
        }
    }

    /// Locks immutable access to a given entry.
    ///
    /// Returns [`None`] if the descriptor is invalid, or the entry is already
    /// mutably locked.
    pub fn get(&self, desc: PoolDescriptor) -> Option<Ref<T>> {
        if !self.is_valid(desc) {
            return None;
        }

        self.list
            .get(desc.index)
            .unwrap()
            .as_ref()
            .unwrap()
            .0
            .try_borrow()
            .ok()
    }

    /// Locks mutable access to a given entry.
    ///
    /// Returns [`None`] if the descriptor is invalid, or there exist other
    /// locks on the entry.
    pub fn get_mut(&self, desc: PoolDescriptor) -> Option<RefMut<T>> {
        if !self.is_valid(desc) {
            return None;
        }

        self.list
            .get(desc.index)
            .unwrap()
            .as_ref()
            .unwrap()
            .0
            .try_borrow_mut()
            .ok()
    }

    pub fn is_valid(&self, desc: PoolDescriptor) -> bool {
        let Some(Some(entry)) = self.list.get(desc.index) else { return false; };
        entry.1 == desc.generation
    }

    fn next_gen(&mut self) -> u64 {
        let current = self.generation;
        self.generation = current.checked_add(1).expect("Pool generation overflow");
        current
    }

    /// Finds an empty pool entry **without claiming it** and advances the
    /// internal allocator index pointer forward.
    ///
    /// This means, that the returned entry must be used.
    ///
    /// If the list is exhausted, it is extended by the value of elements stored
    /// in `self.growth_size`.
    fn next_empty(&mut self) -> (usize, &mut PoolEntry<T>) {
        // The top free can either be within range, or exactly 1 out of bounds
        debug_assert!(self.lowest_free <= self.list.len());

        let index = self
            .list
            .iter_mut()
            .enumerate()
            .skip(self.lowest_free)
            .find(|entry| entry.1.is_none())
            .map(|entry| entry.0);

        if let Some(index) = index {
            self.lowest_free = self
                .list
                .iter()
                .enumerate()
                .skip(index + 1)
                .find(|entry| entry.1.is_none())
                .map(|entry| entry.0)
                .unwrap_or(self.list.len());

            (index, &mut self.list[index])
        } else {
            // The list is full, we need to allocate a new entry
            let index = self.list.len();
            self.lowest_free = index + 1;
            self.list
                .extend(iter::from_fn(|| Some(None)).take(self.growth_size));
            (index, &mut self.list[index])
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PoolDescriptor {
    // TODO: think of changing this to a u32 pair
    pub index: usize,
    pub generation: u64,
}

#[cfg(test)]
mod tests {
    use super::Pool;
    use std::iter;

    #[test]
    pub fn basic_pool_test() {
        const STARTING_SIZE: usize = 16;

        let mut pool: Pool<bool> = Pool::with_capacity(STARTING_SIZE);

        let descriptors = iter::from_fn(|| Some(pool.allocate(false)))
            .take(STARTING_SIZE)
            .collect::<Vec<_>>();

        // Modification checks
        *pool.get_mut(descriptors[5]).unwrap() = true;
        assert_eq!(*pool.get(descriptors[4]).unwrap(), false);
        assert_eq!(*pool.get(descriptors[5]).unwrap(), true);
        assert_eq!(*pool.get(descriptors[6]).unwrap(), false);

        // After-free use check
        pool.free(descriptors[6]);
        assert_eq!(*pool.get(descriptors[5]).unwrap(), true);
        assert!(pool.get(descriptors[6]).is_none());
        assert_eq!(*pool.get(descriptors[7]).unwrap(), false);

        // This allocation should get an index=6, but a different generation
        let current = pool.list.len();
        let in_place_of_6 = pool.allocate(true);
        assert_eq!(current, pool.list.len());
        assert_eq!(in_place_of_6.index, descriptors[6].index);
        assert_ne!(in_place_of_6.generation, descriptors[6].generation);
        assert!(pool.get(descriptors[6]).is_none());
        assert_eq!(*pool.get(in_place_of_6).unwrap(), true);

        // Extension check (uses internal field access)
        let current = pool.list.len();
        let extended_desc_0 = pool.allocate(true);
        let extended_desc_1 = pool.allocate(true);
        assert_ne!(current, pool.list.len());
        assert_ne!(extended_desc_0.index, extended_desc_1.index);

        // Check if extended_desc_X don't collide
        for desc in [extended_desc_0, extended_desc_1] {
            for &other in &descriptors {
                assert_ne!(desc.index, other.index);
            }
        }
    }
}
