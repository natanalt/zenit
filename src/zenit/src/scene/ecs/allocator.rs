// TODO: make the entity allocator actually efficient
//       it is hilariously slow rn

use super::UnmanagedEntity;

pub struct EntityAllocator {
    generations: Vec<Option<usize>>,
    newest_generation: usize,
    top: usize,
}

impl EntityAllocator {
    pub fn new() -> Self {
        Self {
            generations: Vec::with_capacity(super::MIN_ENTITY_CAP),
            newest_generation: 1,
            top: 0,
        }
    }

    pub fn allocate_entity(&mut self) -> UnmanagedEntity {
        let result = UnmanagedEntity {
            index: self.top,
            generation: self.newest_generation,
        };
        self.generations[result.index] = Some(result.generation);

        self.newest_generation += 1;
        self.top += 1;

        loop {
            match self.generations.get_mut(self.top) {
                Some(entity) => {
                    if entity.is_none() {
                        break;
                    }
                }
                None => {
                    self.generations.push(None);
                    break;
                }
            }

            self.top += 1;
        }

        result
    }

    pub fn free_entity(&mut self, ent: UnmanagedEntity) {
        
    }

    #[inline]
    fn ensure_vec_size(&mut self, desired: usize) {
        if self.generations.len() < desired {
            self.generations.resize(desired, None);
        }
    }
}
