use crate::{
    profiler::{FrameProfiler, FrameStage},
    scene::{EngineState, Entity, EntityBuilder, Universe}, render::Renderer,
};
use rustc_hash::FxHashMap;
use std::{
    any::{Any, TypeId},
    time::Duration, rc::Rc,
};
use winit::{event::WindowEvent, window::Window};

pub struct EngineContext {
    /// Window events, filled by event loop code every frame
    pub events: Vec<WindowEvent<'static>>,
    pub globals: FxHashMap<TypeId, Box<dyn Any>>,
    pub profiler: FrameProfiler,
    
    pub universe: Universe,
    pub entities_to_add: Vec<EntityBuilder>,
    pub entities_to_remove: Vec<Entity>,

    pub renderer: Renderer,
}

impl EngineContext {
    pub fn new(window: Rc<Window>) -> Self {
        Self {
            events: Vec::with_capacity(20),
            globals: FxHashMap::default(),
            profiler: FrameProfiler::new(),

            universe: Universe::new(),
            entities_to_add: Vec::with_capacity(32),
            entities_to_remove: Vec::with_capacity(32),

            renderer: Renderer::new(window),
        }
    }

    pub fn with_global(mut self, global: impl Any) -> Self {
        self.globals.insert(global.type_id(), Box::new(global));
        self
    }

    pub fn with_named_global<G>(mut self, global: G::Target) -> Self
    where
        G: NamedData
    {
        self.globals.insert(TypeId::of::<G>(), Box::new(global));
        self
    }

    pub fn run_frame(&mut self) {
        self.profiler.next_stage(FrameStage::SceneProcessing);

        // Process the scene
        for entity in self.universe.iter_entities() {
            let mut storage = self
                .universe
                .get_entity_refcell(entity)
                .unwrap()
                .borrow_mut();

            if let Some(mut behavior) = storage.behavior.take() {
                behavior.process(
                    &mut storage,
                    EngineState {
                        globals: &mut self.globals,
                        events: &self.events,
                        universe: &self.universe,
                        to_add: &mut self.entities_to_add,
                        to_remove: &mut self.entities_to_remove,
                    },
                );
                storage.behavior = Some(behavior);
            }
        }

        for desc in self.entities_to_remove.drain(..) {
            self.universe.free_entity(desc);
        }

        for builder in self.entities_to_add.drain(..) {
            builder.build(&mut self.universe);
        }

        self.profiler.next_stage(FrameStage::RenderProcessing);
        // ...

        self.profiler.finish_frame();
    }
}

/// A type that lets you declare data under a different type name. Useful if
/// you want to store some mundane data in [`SceneState`], like [`f32`], but
/// with an actually decent type-safe name.
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

/// Wrapper for creating blank ZSTs that implement [`NamedData`].
///
/// Example: `declare_named_data!(TimeStep as Duration)`
#[macro_export]
macro_rules! declare_named_data {
    ($name:ident as $target:ty) => {
        pub struct $name;
        impl crate::engine::NamedData for $name {
            type Target = $target;
        }
    };
}

declare_named_data!(TimeStep as Duration);
