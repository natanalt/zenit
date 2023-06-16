use super::{Component, Entity, Universe};

pub struct EntityBuilder<'uni> {
    universe: &'uni mut Universe,
    entity: Entity,
}

impl<'uni> EntityBuilder<'uni> {
    pub fn new(universe: &'uni mut Universe) -> Self {
        Self {
            entity: universe.create_entity(),
            universe,
        }
    }

    pub fn with_component(&mut self, value: impl Component) -> &mut Self {
        self.universe.set_component(self.entity, value);
        self
    }

    pub fn finish(&mut self) -> Entity {
        self.entity
    }
}
