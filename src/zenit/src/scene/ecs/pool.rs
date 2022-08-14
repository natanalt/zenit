use std::{iter, num::NonZeroUsize};

#[derive(Clone, Copy, PartialEq)]
pub struct EntityLocation {
    pub index: usize,
    pub generation: NonZeroUsize,
}

// TODO: improve the pool allocation mechanism?
//       For example try downscaling the pool if there's too much unused space

pub struct EntityPool {
    /// How many more generation entries should be allocated when we run out
    /// of space
    pub resize_delta: usize,

    top_generation: NonZeroUsize,

    /// Contains all generation information for every entity in the pool.
    generations: Vec<Option<NonZeroUsize>>,

    /// Contains a list of all free entries within the generations vector.
    /// If this list is empty, then generations needs to be resized
    free_indices: Vec<usize>,
}

impl EntityPool {
    pub fn new() -> Self {
        // Assuming that by default we'll want to allocate an additional page
        // every time we need more entity space.
        let resize_delta = 4096 / std::mem::size_of::<usize>();
        Self {
            resize_delta,
            top_generation: NonZeroUsize::new(1).unwrap(),
            generations: iter::repeat(None).take(resize_delta).collect(),
            free_indices: (0..resize_delta).into_iter().collect(),
        }
    }

    pub fn allocate_entity(&mut self) -> EntityLocation {
        if self.free_indices.is_empty() {
            self.allocate_next_batch();
        }

        let index = self.free_indices.pop().unwrap();
        let generation = self.next_generation();
        self.generations[index] = Some(generation);

        EntityLocation { index, generation }
    }

    pub fn free_entity(&mut self, ent: EntityLocation) {
        debug_assert!(self.is_valid(ent), "invalid entity location");
        self.generations[ent.index] = None;
        self.free_indices.push(ent.index);
    }

    /// Checks whether given location matches a valid entity
    #[inline]
    pub fn is_valid(&self, ent: EntityLocation) -> bool {
        if let Some(gen_opt) = self.generations.get(ent.index) {
            if let Some(gen) = gen_opt {
                return *gen == ent.generation;
            }
        }
        false
    }

    #[inline]
    fn allocate_next_batch(&mut self) {
        let current_max = self.generations.len();
        let resize_delta = self.resize_delta;

        self.generations
            .extend(iter::repeat(None).take(resize_delta));
        self.free_indices
            .extend(current_max..(current_max + resize_delta));
    }

    #[inline]
    fn next_generation(&mut self) -> NonZeroUsize {
        let gen = self.top_generation;
        self.top_generation = self.top_generation
            .checked_add(1)
            .expect("ran out of generations");
        gen
    }
}
