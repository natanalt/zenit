use crate::render::components::*;
use super::accessor::EntityAccessor;
use super::{Component, components::*, Entity};
use paste::paste;
use std::mem;
use std::{
    any::TypeId,
    iter,
    num::NonZeroU32,
};

// ----------------------------------------------------------------------------
// Any new components must be added here
// ----------------------------------------------------------------------------
crate::create_universe! {
    components {
        // Common
        TransformComponent,
        
        // Renderer
        SceneComponent, RenderComponent, CameraComponent,
    }
}
// ----------------------------------------------------------------------------

/// Amount of [`Universe`] slots to grow by whenever the containers run out of space.
const ECS_GROW_AMOUNT: u32 = 50;

impl Universe {
    /// Allocates a new entity slot.
    pub fn create_entity(&mut self) -> Entity {
        if self.free_indices.len() == 0 {
            self.alloc_blank_indices(ECS_GROW_AMOUNT);
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

    /// Allocates new blank indices in the free index list.
    fn alloc_blank_indices(&mut self, amount: u32) {
        let top_index = self.generations.len() as u32;
        let new_top_index = top_index.checked_add(amount).expect("ECS index overflow??");
        let new_indices = top_index..new_top_index;
        self.free_indices.extend(new_indices.rev());
    }

    pub fn get_components<T: RegisteredComponent>(&self) -> impl Iterator<Item = (Entity, &T)> {
        self.get_component_vec::<T>()
            .backend
            .iter()
            .enumerate()
            .filter_map(|(index, component)| {
                let generation = self.generations[index].unwrap();
                Some((
                    Entity {
                        index: index as u32,
                        generation,
                    },
                    component.as_ref()?,
                ))
            })
    }

    pub fn get_components_mut<T: RegisteredComponent>(
        &mut self,
    ) -> impl Iterator<Item = (Entity, &mut T)> {
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

    pub fn get_component<T: RegisteredComponent>(&self, entity: Entity) -> Option<&T> {
        assert!(self.validate_entity(entity), "invalid entity access");
        self.get_component_vec().backend[entity.index as usize].as_ref()
    }

    pub fn get_component_mut<T: RegisteredComponent>(&mut self, entity: Entity) -> Option<&mut T> {
        assert!(self.validate_entity(entity), "invalid entity access");
        self.get_component_vec_mut().0.backend[entity.index as usize].as_mut()
    }

    pub fn access_entity(&self, entity: Entity) -> EntityAccessor {
        assert!(self.validate_entity(entity), "invalid entity access");
        EntityAccessor {
            universe: self,
            entity,
        }
    }

    pub fn iter_entities(&self) -> impl Iterator<Item = EntityAccessor> {
        self.generations
            .iter()
            .enumerate()
            .filter_map(|(index, generation)| Some(EntityAccessor {
                universe: self,
                entity: Entity {
                    index: index as u32,
                    generation: generation.clone()?,
                },
            }))
    }
}

/// Vector implementation for storing optional components. Backed by a `Vec<Option<T>>`.
/// Its internal functions make it work more like an "infinitely sized" vector.
pub(super) struct ComponentVec<T: RegisteredComponent> {
    // Possible optimization idea for the future:
    //  - Store the items as a list of `MaybeUninit<T>`
    //  - Store presence markers in a separate vector
    //  - This would allow faster scans for specified present entities, as many entities
    //    could be checked at once.
    backend: Vec<Option<T>>,
}

#[allow(dead_code)] // The utility functions are for future reference
impl<T: RegisteredComponent> ComponentVec<T> {
    /// Creates a new [`ComponentVec`].
    pub fn new() -> Self {
        Self { backend: vec![] }
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
    ///
    /// ## Panics
    /// With debug assertions, panics if the component is not present.
    pub fn clear(&mut self, index: u32) {
        if let Some(component) = self.backend.get_mut(index as usize) {
            debug_assert!(component.is_some(), "component is already clear");
            *component = None;
        } else {
            #[cfg(debug_assertions)]
            panic!("component is already clear");
        }
    }

    /// Shrinks the underlying vec's capacity, and cuts off any trailing clear components.
    pub fn optimize(&mut self) {
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
