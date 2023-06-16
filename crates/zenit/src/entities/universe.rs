use super::accessor::{EntityAccessor, EntityAccessorMut};
use super::{Component, Entity};
use ahash::AHashMap;
use std::any::Any;
use std::{any::TypeId, iter, num::NonZeroU32};

/// Amount of [`Universe`] slots to grow by whenever the containers run out of space.
const ECS_GROW_AMOUNT: u32 = 50;

pub struct Universe {
    top_generation: NonZeroU32,
    free_indices: Vec<u32>,
    generations: Vec<Option<NonZeroU32>>,

    vectors: AHashMap<TypeId, Box<dyn ComponentVec>>,
}

impl Universe {
    pub fn new() -> Self {
        Self {
            top_generation: NonZeroU32::new(1).unwrap(),
            free_indices: vec![],
            generations: vec![],
            vectors: AHashMap::default(),
        }
    }

    /// Returns an immutable reference to the specified component vector.
    pub(in crate::entities) fn get_component_vec<T: Component>(&self) -> &ComponentVecImpl<T> {
        match self.vectors.get(&TypeId::of::<T>()) {
            Some(component_vector) => component_vector
                .try_cast()
                .expect("invalid internal ecs type mapping"),
            None => {
                // FIXME: This is a delibirate memleak, it's only a few bytes long, but fucking hell
                //
                // This could be fixed by:
                //  - Rust allowing generic static variables
                //  - Giving ComponentVecImpl a layout that can be reliably const constructed as an empty vector
                Box::leak(Box::new(ComponentVecImpl::new()))
            }
        }
    }

    /// Returns mutable references to the component vector, and the generation vector,
    /// accesss to which would otherwise be blocked by the borrow checker.
    #[inline(always)] // This should optimize out to a simple pointer calculation
    pub(in crate::entities) fn get_component_vec_mut<T: Component>(
        &mut self,
    ) -> (&mut ComponentVecImpl<T>, &mut Vec<Option<NonZeroU32>>) {
        (
            self.vectors
                .entry(TypeId::of::<T>())
                .or_insert_with(|| Box::new(ComponentVecImpl::<T>::new()))
                .try_cast_mut()
                .expect("invalid internal ecs type mapping"),
            &mut self.generations,
        )
    }

    pub fn delete_entity(&mut self, entity: Entity) {
        assert!(
            self.validate_entity(entity),
            "attempting to delete an invalid entity"
        );

        self.generations[entity.index as usize] = None;

        self.free_indices.push(entity.index);
    }

    /// Allocates a new entity slot.
    ///
    /// ## Panics
    ///  - on index overflow
    ///  - on generation overflow
    pub fn create_entity(&mut self) -> Entity {
        if self.free_indices.len() == 0 {
            let top_index = self.generations.len() as u32;
            let new_top_index = top_index
                .checked_add(ECS_GROW_AMOUNT)
                .expect("ECS index overflow??");
            let new_indices = top_index..new_top_index;
            self.free_indices.extend(new_indices.rev());
        }

        let index = self.free_indices.pop().unwrap();
        let generation = self.top_generation;
        self.top_generation = generation
            .checked_add(1)
            .expect("ECS generation overflow??");

        vec_write_with_grow(&mut self.generations, index as usize, Some(generation));

        Entity { index, generation }
    }

    /// Checks whether provided [`Entity`] constitutes a valid handle.
    #[inline]
    pub fn validate_entity(&self, entity: Entity) -> bool {
        if let Some(&generation) = self.generations.get(entity.index as usize) {
            Some(entity.generation) == generation
        } else {
            false
        }
    }

    /// Adds a component to the specified entity. The component is overwritten if it already exists.
    ///
    /// ## Panics
    ///  - if the handle is invalid
    pub fn set_component<T: Component>(&mut self, entity: Entity, value: T) {
        assert!(self.validate_entity(entity), "invalid entity access");
        let (components, _) = self.get_component_vec_mut::<T>();
        components.set(entity.index, value);
    }

    /// Removes a component from the specified entity.
    ///
    /// ## Panics
    ///  - if the handle is invalid
    ///  - if the entity doesn't have the component
    pub fn remove_component<T: Component>(&mut self, entity: Entity) -> T {
        assert!(self.validate_entity(entity), "invalid entity access");
        self.get_component_vec_mut::<T>().0.take(entity.index)
    }

    /// Checks if the entity has a specified component
    ///
    /// ## Panics
    /// - if the handle is invalid
    pub fn has_component<T: Component>(&self, entity: Entity) -> bool {
        assert!(self.validate_entity(entity), "invalid entity access");
        self.get_component_vec::<T>().is_set(entity.index)
    }

    /// Returns an optional component reference of the specified entity
    ///
    /// ## Panics
    /// - if the handle is invalid
    pub fn get_component<T: Component>(&self, entity: Entity) -> Option<&T> {
        assert!(self.validate_entity(entity), "invalid entity access");
        self.get_component_vec().backend[entity.index as usize].as_ref()
    }

    /// Returns an optional component mutable reference of the specified entity
    ///
    /// ## Panics
    /// - if the handle is invalid
    pub fn get_component_mut<T: Component>(&mut self, entity: Entity) -> Option<&mut T> {
        assert!(self.validate_entity(entity), "invalid entity access");
        self.get_component_vec_mut().0.backend[entity.index as usize].as_mut()
    }

    /// Returns an iterator that scans through all valid entities with component `T`.
    pub fn get_components<T: Component>(&self) -> impl Iterator<Item = (EntityAccessor, &T)> {
        self.get_component_vec::<T>()
            .backend
            .iter()
            .enumerate()
            .filter_map(|(index, component)| {
                let generation = self.generations[index].unwrap();
                Some((
                    EntityAccessor {
                        universe: self,
                        entity: Entity {
                            index: index as u32,
                            generation,
                        },
                    },
                    component.as_ref()?,
                ))
            })
    }

    pub fn get_components_mut<T: Component>(&mut self) -> impl Iterator<Item = (Entity, &mut T)> {
        let (components, generations) = self.get_component_vec_mut::<T>();
        components
            .backend
            .iter_mut()
            .enumerate()
            .filter_map(|(index, component)| {
                let generation = generations[index].unwrap();
                Some((
                    Entity {
                        index: index as u32,
                        generation,
                    },
                    component.as_mut()?,
                ))
            })
    }

    pub fn access_entity(&self, entity: Entity) -> EntityAccessor {
        assert!(self.validate_entity(entity), "invalid entity access");
        EntityAccessor {
            universe: self,
            entity,
        }
    }

    pub fn access_entity_mut(&mut self, entity: Entity) -> EntityAccessorMut {
        assert!(self.validate_entity(entity), "invalid entity access");
        EntityAccessorMut {
            universe: self,
            entity,
        }
    }

    pub fn iter_entities(&self) -> impl Iterator<Item = EntityAccessor> {
        self.generations
            .iter()
            .enumerate()
            .filter_map(|(index, generation)| {
                Some(EntityAccessor {
                    universe: self,
                    entity: Entity {
                        index: index as u32,
                        generation: generation.clone()?,
                    },
                })
            })
    }
}

/// Trait representing a type that contains within itself a [`ComponentVecImpl`].
///
/// Its primary purpose is to allow for type erasure to allow calling [`clear`] and [`shrink_to_fit`]
/// without knowing the underlying component type (as is required in some [`Universe`] functions).
///
/// It's a bit hacky, and could be a bit more optimized perhaps, but it does the job.
pub(in crate::entities) trait ComponentVec: Any + Send + Sync {
    fn clear(&mut self, index: u32);
    fn shrink_to_fit(&mut self);

    // This may be unnecessary if rust#65991 (dyn upcasting) gets stabilized
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl dyn ComponentVec {
    pub fn try_cast<T: Component>(&self) -> Option<&ComponentVecImpl<T>> {
        self.as_any().downcast_ref()
    }

    pub fn try_cast_mut<T: Component>(&mut self) -> Option<&mut ComponentVecImpl<T>> {
        self.as_any_mut().downcast_mut()
    }
}

impl<T: Component> ComponentVec for ComponentVecImpl<T> {
    fn clear(&mut self, index: u32) {
        self.clear(index);
    }

    fn shrink_to_fit(&mut self) {
        self.shrink_to_fit();
    }

    fn as_any(&self) -> &dyn Any {
        self as &dyn Any
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self as &mut dyn Any
    }
}

pub(in crate::entities) struct ComponentVecImpl<T: Component> {
    backend: Vec<Option<T>>,
}

impl<T: Component> ComponentVecImpl<T> {
    /// Creates a new [`ComponentVecImpl`].
    pub const fn new() -> Self {
        Self {
            backend: Vec::new(),
        }
    }

    /// Returns a reference to the component at specified index. Returns [`None`], if the index
    /// is out of range, or the component is cleared.
    pub fn get(&self, index: u32) -> Option<&T> {
        self.backend
            .get(index as usize)
            .map(Option::as_ref)
            .flatten()
    }

    /// Like `ComponentVec::get`, but returns a mutable reference.
    pub fn get_mut(&mut self, index: u32) -> Option<&mut T> {
        self.backend
            .get_mut(index as usize)
            .map(Option::as_mut)
            .flatten()
    }

    /// Sets a specified component, growing the underlying vector if necessary.
    pub fn set(&mut self, index: u32, value: T) {
        let index = index as usize;
        if index < self.backend.len() {
            self.backend[index] = Some(value);
        } else {
            let required = index - self.backend.len() + 1;
            self.backend.extend(
                iter::repeat_with(|| None)
                    .take(required - 1)
                    .chain(iter::once(Some(value))),
            );
        }
    }

    /// Checks if the given component is present in the vector.
    pub fn is_set(&self, index: u32) -> bool {
        self.backend
            .get(index as usize)
            .map(Option::is_some)
            .unwrap_or(false)
    }

    /// Clears the specified component.
    pub fn clear(&mut self, index: u32) {
        if let Some(component) = self.backend.get_mut(index as usize) {
            *component = None;
        }
    }

    /// Takes ane clears the specified component.
    ///
    /// ## Panics
    /// Panics if the component doesn't exist.
    pub fn take(&mut self, index: u32) -> T {
        self.backend
            .get_mut(index as usize)
            .and_then(Option::take)
            .expect("component doesn't exist")
    }

    /// Shrinks the underlying vec's capacity, and cuts off any trailing clear components.
    pub fn shrink_to_fit(&mut self) {
        let trailing_clears = self
            .backend
            .iter()
            .rev()
            .take_while(|x| x.is_none())
            .count();

        self.backend.truncate(self.backend.len() - trailing_clears);
        self.backend.shrink_to_fit();
    }
}

fn vec_write_with_grow<T: Default>(vec: &mut Vec<T>, index: usize, value: T) {
    if index >= vec.len() {
        let missing = index - vec.len() + 1;
        vec.extend(iter::from_fn(|| Some(T::default())).take(missing));
    }
    vec[index] = value;
}
