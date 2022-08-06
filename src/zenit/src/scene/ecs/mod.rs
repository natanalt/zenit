use self::allocator::EntityAllocator;
use std::sync::Arc;

mod allocator;

/// Minimal capacity for various universe vectors
const MIN_ENTITY_CAP: usize = 64;

// the name is trying to be special
pub struct Universe {
    allocator: EntityAllocator,
}

impl Universe {
    pub fn new() -> Self {
        Self {
            allocator: EntityAllocator::new(),
        }
    }

    pub fn allocate_entity(&self) -> Entity {
        todo!()
        //self.allocator.allocate_entity().to_managed(universe)
    }

    pub fn free_entity(&self, entity: Entity) {
        todo!()
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct UnmanagedEntity {
    pub index: usize,
    pub generation: usize,
}

impl UnmanagedEntity {
    pub fn to_managed(self, universe: Arc<Universe>) -> Entity {
        Entity::from_unmanaged(universe, self)
    }
}

#[derive(Clone)]
pub struct Entity {
    pub universe: Arc<Universe>,
    pub index: usize,
    pub generation: usize,
}

impl Entity {
    /// Assumes that given uni is valid, as there are no checks to prove it otherwise
    pub fn from_unmanaged(universe: Arc<Universe>, ent: UnmanagedEntity) -> Self {
        Self {
            universe,
            index: ent.index,
            generation: ent.generation,
        }
    }
}

impl Into<UnmanagedEntity> for Entity {
    fn into(self) -> UnmanagedEntity {
        UnmanagedEntity {
            index: self.index,
            generation: self.generation,
        }
    }
}

impl PartialEq for Entity {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && self.generation == other.generation
    }
}

pub trait Component {
    fn process(&mut self, parent: &Entity);
}
