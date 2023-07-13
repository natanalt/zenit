use std::num::NonZeroU32;

/// Untyped handle for a [`Pool`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PoolHandle {
    pub index: u32,
    pub generation: NonZeroU32,
}

/// Generic pool implementation. Allocates values of type `T`, and allows accessing them via
/// dedicated handles. The handles include 32-bit index and generation counts, the latter
/// being used as a simple use-after-free test.
///
/// Note, this implementation isn't panic-proof. There are some conditions that can cause panics:
///  * overflowing the 32-bit index counter
///  * overflowing the 32-bit generation counter (reached after allocating over 4 294 967 294
///    entries total)
///  * improper handle accesses in `get` or `get_mut` (`try_*` variants exist)
///
/// ## Example
/// ```
/// # use zenit_utils::Pool;
///
/// let mut pool: Pool<u32> = Pool::new();
///
/// // Allocate a new pool element with a value of 10
/// let handle = pool.allocate(10);
/// assert_eq!(*pool.get(handle), 10);
///
/// // Deallocate it - the handle becomes invalid
/// pool.deallocate(handle);
/// assert!(pool.try_get(handle).is_none());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pool<T> {
    top_generation: NonZeroU32,
    free_indices: Vec<u32>,
    generations: Vec<Option<NonZeroU32>>,
    values: Vec<Option<T>>,

    /// The amount of entries internal vectors should grow by.
    /// Small `T` values should keep it high, big `T` values should keep it low.
    /// The default from [`Pool::new`] is 10.
    pub growth_amount: NonZeroU32,
}

impl<T> Pool<T> {
    /// Creates a new pool.
    pub fn new() -> Self {
        Self {
            top_generation: NonZeroU32::new(1).unwrap(),
            free_indices: vec![],
            generations: vec![],
            values: vec![],
            growth_amount: NonZeroU32::new(10).unwrap(),
        }
    }

    /// Allocates a new pool entry, fills it with `value`, and returns its handle.
    ///
    /// ## Panics
    ///  * On 32-bit index overflow
    ///  * On 32-bit generation overflow
    pub fn allocate(&mut self, value: T) -> PoolHandle {
        let index = match self.free_indices.pop() {
            Some(index) => index,
            None => {
                // Out of indices, grow all vecs
                let low_index = self.generations.len() as u32;
                let high_index = low_index
                    .checked_add(self.growth_amount.get())
                    .expect("pool index overflow");
                let growth_range = low_index..high_index;

                // Reverse index range, so that pop gets lowest entries
                self.free_indices.extend(growth_range.clone().rev());
                self.generations.extend(growth_range.clone().map(|_| None));
                self.values.extend(growth_range.map(|_| None));

                self.free_indices.pop().unwrap()
            }
        };

        let generation = self.top_generation;
        self.top_generation = generation.checked_add(1).expect("pool generation overflow");

        self.values[index as usize] = Some(value);
        self.generations[index as usize] = Some(generation);

        PoolHandle { index, generation }
    }

    /// Deallocates a specified pool entry, dropping the held value.
    ///
    /// ## Panics
    /// Panics if the handle is invalid.
    pub fn deallocate(&mut self, handle: PoolHandle) {
        assert!(self.is_valid(handle));
        self.free_indices.push(handle.index);
        self.generations[handle.index as usize] = None;
        self.values[handle.index as usize] = None;
    }

    /// Returns an immutable reference to a specified pool entry.
    ///
    /// ## Panics
    /// Panics if the handle is invalid.
    pub fn get(&self, handle: PoolHandle) -> &T {
        assert!(self.is_valid(handle));
        self.values[handle.index as usize].as_ref().unwrap()
    }

    /// Returns an immutable reference to a specified pool entry. If the handle is invalid, [`None`]
    /// is returned.
    pub fn try_get(&self, handle: PoolHandle) -> Option<&T> {
        self.is_valid(handle).then(|| self.get(handle))
    }

    /// Returns a mutable reference to a specified pool entry.
    ///
    /// ## Panics
    /// Panics if the handle is invalid.
    pub fn get_mut(&mut self, handle: PoolHandle) -> &mut T {
        assert!(self.is_valid(handle));
        self.values[handle.index as usize].as_mut().unwrap()
    }

    /// Returns a mutable reference to a specified pool entry. If the handle is invalid, [`None`]
    /// is returned.
    pub fn try_get_mut(&mut self, handle: PoolHandle) -> Option<&mut T> {
        self.is_valid(handle).then(|| self.get_mut(handle))
    }

    /// Verifies the validity of the specified handle.
    #[inline]
    pub fn is_valid(&self, handle: PoolHandle) -> bool {
        self.generations
            .get(handle.index as usize)
            .map(|&generation| generation == Some(handle.generation))
            .unwrap_or(false)
    }

    /// Counts how many pool entries are occupied.
    ///
    /// This is a fairly expensive operation, as it linearly scans all entries.
    pub fn count_allocated(&self) -> u32 {
        self.generations
            .iter()
            .filter(|gen| gen.is_some())
            .count()
            .try_into()
            .unwrap()
    }
}

impl<T> Default for Pool<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::Pool;

    #[test]
    pub fn generic_pool_test() {
        const VALUE_A: u32 = 123;
        const VALUE_B: u32 = 456;
        const VALUE_C: u32 = 789;
        const VALUE_D: u32 = 789;

        let mut pool = Pool::new();
        let a = pool.allocate(VALUE_A);
        let b = pool.allocate(VALUE_B);
        let c = pool.allocate(VALUE_C);

        assert_eq!(pool.count_allocated(), 3);
        assert_eq!(*pool.get(a), VALUE_A);
        assert_eq!(*pool.get(b), VALUE_B);
        assert_eq!(*pool.get(c), VALUE_C);

        pool.deallocate(b);
        assert!(pool.try_get(b).is_none());

        assert_eq!(pool.count_allocated(), 2);
        assert_eq!(*pool.get(a), VALUE_A);
        assert_eq!(*pool.get(c), VALUE_C);

        let d = pool.allocate(VALUE_D);

        assert_eq!(pool.count_allocated(), 3);
        assert_eq!(*pool.get(a), VALUE_A);
        assert_eq!(*pool.get(c), VALUE_C);
        assert_eq!(*pool.get(d), VALUE_D);
    }
}
