//! # The ECS scene system
//! This module contains everything related to scenes, entities, components
//! and all the fanciness. This is what puts the engine into motion and talks
//! with other systems to do ✨ stuff ✨

use crate::engine::NamedData;
use rustc_hash::{FxHashMap, FxHashSet};
use std::{
    any::{Any, TypeId},
    cell::{RefCell, RefMut},
    convert::identity,
    iter,
};
use thiserror::Error;
use winit::event::WindowEvent;

pub mod blank;

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

    pub fn builder() -> EntityBuilder {
        EntityBuilder::new()
    }
}

pub struct Universe {
    top_generation: usize,
    new_search_start: usize,
    entity_count: usize,
    entities: Vec<Option<(usize, RefCell<EntityStorage>)>>,
}

impl Default for Universe {
    fn default() -> Self {
        Self::new()
    }
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

    /// Borrows this universe immutably and begins iterating through all
    /// currently existing entity descriptors
    pub fn iter_entities(&self) -> impl Iterator<Item = Entity> + '_ {
        self.entities
            .iter()
            .map(Option::as_ref)
            .filter_map(identity)
            .enumerate()
            .map(|(index, (generation, _))| Entity {
                index,
                generation: *generation,
            })
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
    /// Called every frame for the behavior
    fn process(&mut self, entity: &mut EntityStorage, scene: EngineState) {
        let _ = entity;
        let _ = scene;
    }
}

/// Marker trait for entity tags
pub trait Tag: Any {}

pub struct EntityStorage {
    pub label: Option<String>,
    descriptor: Entity,
    tags: RefCell<FxHashSet<TypeId>>,

    /// Access to this field is not available during processing.
    /// It should **only** be used by scene processing code, and absolutely
    /// nothing else.
    pub behavior: Option<Box<dyn EntityBehavior>>,
}

impl EntityStorage {
    pub fn descriptor(&self) -> Entity {
        self.descriptor
    }

    pub fn add_tag<T: Tag>(&self) -> bool {
        self.tags.borrow_mut().insert(TypeId::of::<T>())
    }

    pub fn has_tag<T: Tag>(&self) -> bool {
        self.tags.borrow().contains(&TypeId::of::<T>())
    }

    pub fn remove_tag<T: Tag>(&self) -> bool {
        self.tags.borrow_mut().remove(&TypeId::of::<T>())
    }
}

/// Builds entities. Can be instantiated through `EntityBuilder::new()` or
/// `Entity::builder()` depending on preference.
#[derive(Default)]
pub struct EntityBuilder {
    label: Option<String>,
    behavior: Option<Box<dyn EntityBehavior>>,
    tags: FxHashSet<TypeId>,
}

impl EntityBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn label(mut self, label: impl ToString) -> Self {
        self.label = Some(label.to_string());
        self
    }

    pub fn behavior(mut self, behavior: impl EntityBehavior) -> Self {
        self.behavior = Some(Box::new(behavior));
        self
    }

    pub fn tag<T: Tag>(mut self) -> Self {
        self.tags.insert(TypeId::of::<T>());
        self
    }

    /// Finalizes the entity bnuild, and properly inserts it into the
    /// specified [`Universe`]
    pub fn build(self, universe: &mut Universe) -> Entity {
        universe.insert_entity(EntityStorage {
            label: self.label,
            descriptor: Entity::INVALID,
            tags: RefCell::new(self.tags),
            behavior: self.behavior,
        })
    }
}

/// Publicly available state data
pub struct EngineState<'a> {
    pub globals: &'a mut FxHashMap<TypeId, Box<dyn Any>>,
    pub events: &'a [WindowEvent<'static>],
    pub universe: &'a Universe,
    pub to_add: &'a mut Vec<EntityBuilder>,
    pub to_remove: &'a mut Vec<Entity>,
}

impl<'a> EngineState<'a> {
    pub fn queue_add(&mut self, builder: EntityBuilder) {
        self.to_add.push(builder);
    }

    pub fn queue_remove(&mut self, desc: Entity) {
        self.to_remove.push(desc);
    }

    // a bit boilerplate-y, but whatevs

    pub fn global<T: Any>(&self) -> &T {
        self.globals
            .get(&TypeId::of::<T>())
            .unwrap()
            .downcast_ref()
            .expect("internal type mistmatch")
    }

    pub fn global_mut<T: Any>(&mut self) -> &mut T {
        self.globals
            .get_mut(&TypeId::of::<T>())
            .unwrap()
            .downcast_mut()
            .expect("internal type mismatch")
    }

    pub fn copy_global<T: Any + Copy>(&self) -> T {
        self.global::<T>().clone()
    }

    pub fn nglobal<T: NamedData>(&self) -> &T::Target {
        self.globals
            .get(&TypeId::of::<T>())
            .unwrap()
            .downcast_ref()
            .expect("internal type mismatch")
    }

    pub fn nglobal_mut<T: NamedData>(&mut self) -> &mut T::Target {
        self.globals
            .get_mut(&TypeId::of::<T>())
            .unwrap()
            .downcast_mut()
            .expect("internal type mismatch")
    }

    pub fn copy_nglobal<T>(&self) -> T::Target
    where
        T: NamedData,
        T::Target: Copy,
    {
        self.nglobal::<T>().clone()
    }

    /// Inserts new data into the state, overwriting any old values that may or
    /// may not have been already present.
    pub fn insert_global<T: Any>(&mut self, value: T) {
        self.globals.insert(TypeId::of::<T>(), Box::new(value));
    }

    /// Inserts new data into the state, overwriting any old values that may or
    /// may not have been already present.
    pub fn insert_nglobal<T: NamedData>(&mut self, value: T::Target) {
        self.globals.insert(TypeId::of::<T>(), Box::new(value));
    }
}
