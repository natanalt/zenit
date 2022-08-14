use self::{pool::{EntityLocation, EntityPool}, component::Components};
use std::{cell::RefCell, num::NonZeroUsize, rc::Rc};

pub mod component;
mod pool;

// the name is trying to be special
pub struct Universe {
    pool: RefCell<EntityPool>,
    components: RefCell<Components>,
}

impl Universe {
    pub fn new() -> Self {
        Self {
            pool: RefCell::new(EntityPool::new()),
            components: RefCell::new(Components::new()),
        }
    }
}

/// Trait for universe functions for Rc<Universe>
pub trait UniverseTrait {
    fn allocate_entity(&self) -> Entity;
    fn free_entity(&self, ent: Entity);
}

impl UniverseTrait for Rc<Universe> {
    fn allocate_entity(&self) -> Entity {
        let loc = self.pool.borrow_mut().allocate_entity();
        Entity {
            universe: self.clone(),
            index: loc.index,
            generation: loc.generation,
        }
    }

    fn free_entity(&self, ent: Entity) {
        self.pool.borrow_mut().free_entity(EntityLocation {
            index: ent.index,
            generation: ent.generation,
        })
    }
}

#[derive(Clone)]
pub struct Entity {
    pub universe: Rc<Universe>,
    pub index: usize,
    pub generation: NonZeroUsize,
}

impl Entity {
    /// Extracts the location out of this entity
    #[inline]
    pub fn location(&self) -> EntityLocation {
        EntityLocation {
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
