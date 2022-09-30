//! # The ECS scene system
//! This module contains everything related to scenes, entities, components
//! and all the fanciness. This is what puts the engine into motion and talks
//! with other systems to do ✨ stuff ✨

use self::ecs::{Entity, EntityBuilder, Universe};
use crate::engine::system::{System, SystemContext};
use rustc_hash::FxHashMap;
use std::{
    any::{Any, TypeId},
    rc::Rc,
    time::Duration,
};
use zenit_proc::HasSystemInterface;
use zenit_utils::ThreadCell;

pub mod ecs;

#[derive(HasSystemInterface)]
pub struct SceneSystem {
    scene_state: ThreadCell<SceneState>,
}

impl Default for SceneSystem {
    fn default() -> Self {
        Self {
            scene_state: ThreadCell::invalid(),
        }
    }
}

#[macro_export]
macro_rules! declare_named_data {
    ($name:ident as $target:ty) => {
        pub struct $name;
        impl crate::scene::NamedData for $name {
            type Target = $target;
        }
    };
}

declare_named_data!(FrameTime as Duration);

impl<'ctx> System<'ctx> for SceneSystem {
    fn name(&self) -> &str {
        "Scene System"
    }

    fn init<'a>(&mut self, _context: &mut SystemContext<'ctx, 'a>) {
        // Now that we're in the system thread, we can initialize the fields,
        // which we could not do in the constructor.
        self.scene_state = ThreadCell::new(SceneState::default());
    }

    fn frame<'a>(&mut self, _context: &mut SystemContext<'ctx, 'a>) {
        let mut state = self.scene_state.get_mut().unwrap();

        {
            let uni = state.universe.clone();
            for entity in uni.iter_entities() {
                let mut storage = uni
                    .get_entity_refcell(entity)
                    .expect("invalid entity descriptor during iteration")
                    .try_borrow_mut()
                    .expect("entity still locked during frame processing?");

                // Locking behavior shouldn't fail, as it is only ever done by
                // this code.
                let behavior = storage.behavior.clone();
                if let Some(behavior) = behavior {
                    behavior
                        .try_borrow_mut()
                        .unwrap()
                        .process(&mut storage, &mut state);
                }
            }
        }

        // The universe shouldn't be cloned by anything else, we're good
        let uni_mut = Rc::get_mut(&mut state.universe).unwrap();
        for entity in state.to_remove.drain(0..) {
            uni_mut.free_entity(entity);
        }
        for builder in state.to_add.drain(0..) {
            builder.build(uni_mut);
        }
    }
}

impl Drop for SceneSystem {
    fn drop(&mut self) {
        self.scene_state
            .clear()
            .expect("SceneSystem dropped from an invalid thread");
    }
}

/// A type that lets you declare data under a different type name. Useful if
/// you want to store some mundane data in [`SceneState`], like [`f32`], but
/// with an actually decent name.
///
/// ## Example
/// ```ignore
/// struct FrameTime;
/// impl NamedData for FrameTime {
///     type Target = std::time::Duration;
/// }
///
/// scene_state.insert_named_data::<FrameTime>(Duration::from_secs_f32(0.016));
/// let ft = scene_state
///     .copy_named_data::<FrameTime>()
///     .unwrap();
/// ```
pub trait NamedData: Any {
    type Target: Any;
}

/// Publicly available state data
#[derive(Default)]
pub struct SceneState {
    data: FxHashMap<TypeId, Box<dyn Any>>,
    universe: Rc<Universe>,

    to_add: Vec<EntityBuilder>,
    to_remove: Vec<Entity>,
}

impl SceneState {
    pub fn queue_add(&mut self, builder: EntityBuilder) {
        self.to_add.push(builder);
    }

    pub fn queue_remove(&mut self, desc: Entity) {
        self.to_remove.push(desc);
    }

    pub fn universe(&self) -> &Universe {
        &self.universe
    }

    // a bit boilerplate-y, but whatevs

    pub fn data<T: Any>(&self) -> &T {
        self.data
            .get(&TypeId::of::<T>())
            .unwrap()
            .downcast_ref()
            .expect("internal type mistmatch")
    }

    pub fn data_mut<T: Any>(&mut self) -> &mut T {
        self.data
            .get_mut(&TypeId::of::<T>())
            .unwrap()
            .downcast_mut()
            .expect("internal type mismatch")
    }

    pub fn copy_data<T: Any + Copy>(&self) -> T {
        self.data::<T>().clone()
    }

    pub fn ndata<T: NamedData>(&self) -> &T::Target {
        self.data
            .get(&TypeId::of::<T>())
            .unwrap()
            .downcast_ref()
            .expect("internal type mismatch")
    }

    pub fn ndata_mut<T: NamedData>(&mut self) -> &mut T::Target {
        self.data
            .get_mut(&TypeId::of::<T>())
            .unwrap()
            .downcast_mut()
            .expect("internal type mismatch")
    }

    pub fn copy_ndata<T>(&self) -> T::Target
    where
        T: NamedData,
        T::Target: Copy,
    {
        self.ndata::<T>().clone()
    }

    /// Inserts new data into the state, overwriting any old values that may or
    /// may not have been already present.
    pub fn insert_data<T: Any>(&mut self, value: T) {
        self.data.insert(TypeId::of::<T>(), Box::new(value));
    }

    /// Inserts new data into the state, overwriting any old values that may or
    /// may not have been already present.
    pub fn insert_ndata<T: NamedData>(&mut self, value: T::Target) {
        self.data.insert(TypeId::of::<T>(), Box::new(value));
    }
}
