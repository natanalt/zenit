use super::{Component, Entity, Universe};

/// Wrapper for accessing components from a single entity inside a universe.
/// For multiple accesses to a single entity using it is generally better, as any validation checks
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

    pub fn get_component<T: Component>(&self) -> Option<&T> {
        self.universe.get_component_vec().get(self.entity.index)
    }

    pub fn has_component<T: Component>(&self) -> bool {
        self.universe
            .get_component_vec::<T>()
            .is_set(self.entity.index)
    }
}

/// Same as [`EntityAccessor`], but allows for mutation.
pub struct EntityAccessorMut<'uni> {
    pub(super) universe: &'uni mut Universe,
    pub(super) entity: Entity,
}

impl<'uni> EntityAccessorMut<'uni> {
    pub fn universe(&'uni mut self) -> &'uni mut Universe {
        self.universe
    }

    pub fn entity(&self) -> Entity {
        self.entity
    }

    pub fn get_component<T: Component>(&self) -> Option<&T> {
        self.universe.get_component_vec().get(self.entity.index)
    }

    pub fn get_component_mut<T: Component>(&mut self) -> Option<&mut T> {
        self.universe
            .get_component_vec_mut()
            .0
            .get_mut(self.entity.index)
    }

    pub fn has_component<T: Component>(&self) -> bool {
        self.universe
            .get_component_vec::<T>()
            .is_set(self.entity.index)
    }

    pub fn add_component(&mut self, value: impl Component) {
        self.universe.set_component(self.entity, value);
    }

    pub fn remove_component<T: Component>(&mut self) -> T {
        self.universe
            .get_component_vec_mut()
            .0
            .take(self.entity.index)
    }
}
