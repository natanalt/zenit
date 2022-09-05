use std::{
    any::{Any, TypeId},
    cell::{RefCell, RefMut},
    collections::HashMap,
    iter,
};
use thiserror::Error;

use super::SceneState;

/// An entity descriptor, a pointer that can be cheaply copied, with a size of
/// 2 pointer sized integers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity {
    /// Index to this entity within the [`Universe`]'s entity array.
    pub index: usize,
    /// Generation value, unique for each entity. Protects against
    /// use-after-free accesses.
    pub generation: usize,
}

impl Entity {
    /// An invalid entity descriptor.
    pub const INVALID: Entity = Entity {
        index: usize::MAX,
        generation: usize::MAX,
    };
}

pub struct Universe {
    top_generation: usize,
    new_search_start: usize,
    entities: Vec<Option<(usize, RefCell<EntityStorage>)>>,
    entity_count: usize,
}

impl Universe {
    /// Creates a blank [`Universe`].
    pub fn new() -> Self {
        Self {
            top_generation: 1,
            new_search_start: 0,
            entities: Vec::with_capacity(100),
            entity_count: 0,
        }
    }

    /// Returns specified entity's reference cell and performs proper entity
    /// descriptor verification.
    pub fn get_entity_refcell(
        &self,
        desc: Entity,
    ) -> Result<&RefCell<EntityStorage>, EntityVerificationError> {
        let (generation, cell) = self
            .entities
            .get(desc.index)
            .ok_or(EntityVerificationError::BadIndex)?
            .as_ref()
            .ok_or(EntityVerificationError::BadIndex)?;

        if *generation != desc.generation {
            return Err(EntityVerificationError::BadGeneration);
        }

        Ok(cell)
    }

    /// Performs verification of specified entity descriptor.
    pub fn verify_entity(&self, desc: Entity) -> Result<(), EntityVerificationError> {
        let _ = self.get_entity_refcell(desc)?;
        Ok(())
    }

    /// Mutably locks specified entity and returns the handle to it.
    ///
    /// ## Panics
    /// Panics if the entity is already locked, or the entity descriptor is
    /// invalid.
    pub fn get(&self, desc: Entity) -> RefMut<'_, EntityStorage> {
        self.get_entity_refcell(desc)
            .expect("invalid entity")
            .borrow_mut()
    }

    /// Deallocates specified entity.
    ///
    /// ## Panics
    /// Panics if the entity is invalid.
    pub fn free_entity(&mut self, desc: Entity) {
        self.verify_entity(desc).expect("invalid entity");
        self.entity_count -= 1;
        self.entities[desc.index] = None;

        if desc.index < self.new_search_start {
            self.new_search_start = desc.index;
        }
    }

    /// Allocates a new entity and inserts it into self, while also updating
    /// the provided [`EntityStorage`] appropriately.
    ///
    /// Not quite designed for general use, consider [`EntityBuilder`] instead.
    ///
    /// ## Panics
    ///  * may panic on OOM, unlikely to happen
    ///  * will panic on generation overflow, will not happen in a million years
    pub fn insert_entity(&mut self, mut storage: EntityStorage) -> Entity {
        let index = self.allocate_empty_entity_index();
        let generation = self.allocate_new_generation();

        let descriptor = Entity { index, generation };

        storage.descriptor = descriptor;
        self.entities[index] = Some((generation, RefCell::new(storage)));

        self.entity_count += 1;

        descriptor
    }

    fn allocate_new_generation(&mut self) -> usize {
        let result = self.top_generation;
        self.top_generation = self
            .top_generation
            .checked_add(1)
            .expect("generation overflow? 😳😳😳");
        result
    }

    fn allocate_empty_entity_index(&mut self) -> usize {
        // TODO: make the entity index allocation algorithm not suck

        // Find the first None entry
        let top = self
            .entities
            .iter()
            .skip(self.new_search_start)
            .enumerate()
            .find(|(_, ent)| ent.is_none())
            .map(|(index, _)| index);

        if let Some(top) = top {
            // Find the new top, or mark it as needing reallocation
            self.new_search_start = self
                .entities
                .iter()
                .skip(top + 1)
                .enumerate()
                .find(|(_, ent)| ent.is_none())
                .map(|(index, _)| index)
                .unwrap_or(usize::MAX);
            top
        } else {
            // We ran out of entity spots, allocate more
            let result = self.entities.len();
            self.entities.extend(iter::from_fn(|| None).take(100));
            self.new_search_start = result + 1;
            result
        }
    }
}

#[derive(Debug, Error)]
pub enum EntityVerificationError {
    #[error("bad entity descriptor (invalid index)")]
    BadIndex,
    #[error("bad entity descriptor (invalid generation)")]
    BadGeneration,
}

pub trait EntityBehavior: Any {
    fn process(&mut self, entity: &mut EntityStorage, scene: &mut SceneState) {
        let _ = entity;
        let _ = scene;
    }
}

pub struct HeldComponent {}

pub struct EntityStorage {
    pub label: Option<String>,
    descriptor: Entity,
    components: HashMap<TypeId, HeldComponent>,
}

impl EntityStorage {
    pub fn descriptor(&self) -> Entity {
        self.descriptor
    }
}

pub struct EntityBuilder {
    label: Option<String>,
}

impl EntityBuilder {
    pub fn new() -> Self {
        Self { label: None }
    }

    pub fn label(mut self, label: impl ToString) -> Self {
        self.label = Some(label.to_string());
        self
    }

    pub fn build(mut self, universe: &mut Universe) -> Entity {
        todo!()
        //universe.insert_entity(EntityStorage {})
    }
}
