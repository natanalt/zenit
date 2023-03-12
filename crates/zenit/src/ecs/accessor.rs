use super::{Universe, Entity, RegisteredComponent};

/// Wrapper for accessing components from a single entity inside a universe.
/// For multiple accesses to a single entity using it is generally faster, as any validation checks
/// can only be performed once.
pub struct EntityAccessor<'uni> {
    pub(super) universe: &'uni Universe,
    pub(super) entity: Entity,
}

impl<'uni> EntityAccessor<'uni> {
    pub fn universe(&self) -> &'uni Universe {
        self.universe
    }

    pub fn entity(&self) -> Entity {
        self.entity
    }

    pub fn get_component<T: RegisteredComponent>(&self) -> Option<&T> {
        self.universe.get_component_vec().get(self.entity.index)
    }

    pub fn has_component<T: RegisteredComponent>(&self) -> bool {
        self.universe.get_component_vec::<T>().is_set(self.entity.index)
    }
}
