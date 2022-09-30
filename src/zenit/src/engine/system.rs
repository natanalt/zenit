//! Systems - separated engine modules running on dedicated threads

use super::{DataTypeMap, SystemInterfaceTypeMap, SystemSync, TypeErasedData};
use std::{
    any::{Any, TypeId},
    sync::{
        atomic::{AtomicBool, Ordering},
        RwLock,
    },
};
use winit::event::WindowEvent;

/// Implements an engine system.
///
/// A system is a module which implements a certain piece of functionality,
/// like rendering, physics, or the overall entity/scene system. Each system
/// runs in a dedicated thread.
///
/// Systems perform their work each frame and then wait for other systems
/// to finish whatever they were doing using provided [`Barrier`] objects,
/// see [`SystemContext`] for details.
///
/// The `'ctx` lifetime refers to all the control flags, barriers, lists, as
/// passed to the systems via [`SystemContext`]. These resources are available
/// for the duration of the system's entire lifetime. The `'local` lifetime
/// refers to any resources available only for the duration of the system
/// callbacks (such any locked resources)
///
/// See the example below for a sample system implementation.
///
/// ## Example
/// ```ignore
///
/// // The derive macro adds a `()` system interface trait implementation
/// #[derive(HasSystemInterface)]
/// struct ExampleSystem;
///
/// impl<'ctx> System<'ctx> for ExampleSystem {
///     fn name(&self) -> &str {
///         "Example System"
///     }
///     
///     fn frame<'a>(&mut self, context: &mut SystemContext<'ctx, 'a>) {
///         // Do frame stuff
///         // ...
///
///         // And optionally you can do stuff in the post frame phase.
///         // You don't have to do this, if this function isn't called,
///         // the controller will do it for you.
///         context.finish_frame_phase();
///         // ...
///     }
/// }
/// ```
pub trait System<'ctx>
where
    Self: Any + Send + Sync,
{
    /// Returns this system's name. Keep it constant.
    fn name(&self) -> &str;

    /// Called before the [`System::frame`] callback of this system, and only
    /// once, after it's added.
    fn init<'local>(&mut self, context: &mut SystemContext<'ctx, 'local>) {
        let _ = context;
    }

    /// Runs the system through a frame, within its dedicated thread.
    fn frame<'local>(&mut self, context: &mut SystemContext<'ctx, 'local>);
}

/// Some systems, for example the renderer, implement a public interface. This
/// trait can specify what kinda system interface this system implements.
pub trait HasSystemInterface: Any + Send + Sync {
    /// The system interface itself. It's exposed to other systems as an
    /// immutable reference, hence its need to be [`Sync`].
    type SystemInterface: Any + Send + Sync;
    /// Creates the system interface. This function will only be called once
    /// on a single instance of this System. If it's ever called more than
    /// once, it's a bug.
    fn create_system_interface(&self) -> Self::SystemInterface;
}

/// Context passed to each system thread, which allows for communication with
/// the engine.
pub struct SystemContext<'ctx, 'local> {
    /// Specifies whether the game loop should run
    pub should_run: &'ctx AtomicBool,

    /// [`SystemSync`] instance for this system
    pub system_sync: &'local mut SystemSync<'ctx>,

    /// All system interfaces
    pub(super) system_interfaces: &'ctx SystemInterfaceTypeMap,

    /// All data available this frame
    pub(super) data: &'local DataTypeMap,

    /// Data types to be removed after this frame.
    pub(super) data_to_remove: &'ctx RwLock<Vec<TypeId>>,

    /// Data types to be added after this frame.
    pub(super) data_to_add: &'ctx RwLock<Vec<(TypeId, TypeErasedData)>>,

    /// All events caught this frame, with the exception of scale factor changes,
    /// since `WindowEvent<'static>` is unable to really store them
    pub events: &'local [WindowEvent<'static>],
}

impl<'ctx, 'local> SystemContext<'ctx, 'local> {
    /// Returns a reference to specified global data
    pub fn data<D: Any + Send + Sync>(&self) -> &'local D {
        self.data
            .get(&TypeId::of::<D>())
            .expect("trying to access unregistered data")
            .downcast_ref::<D>()
            .expect("invalid data T->V mapping")
    }

    /// Returns a reference to specified system's interface
    pub fn system<S: HasSystemInterface>(&self) -> &'ctx S::SystemInterface {
        self.system_interfaces
            .get(&TypeId::of::<S::SystemInterface>())
            .map(|si| si as &dyn Any)
            .expect("trying to access unregistered system's interface")
            .downcast_ref::<S::SystemInterface>()
            .expect("invalid sysint T->V mapping")
    }

    /// Awaits finishing of frame phase, allowing to perform operations in the
    /// post-frame phase. Must be called only once. If a system doesn't call it,
    /// it'll be called automatically by the system controller.
    pub fn finish_frame_phase(&mut self) {
        todo!()
    }

    /// Enqueues a specific data value to be added into the global engine list.
    /// The modification will happen next frame, and will overwrite any existing
    /// data.
    pub fn queue_add<D: Any + Send + Sync>(&mut self, value: impl Into<Box<D>>) {
        let tid = TypeId::of::<D>();
        self.data_to_add.write().unwrap().push((tid, value.into()));
    }

    /// Enqueues removal of specific data
    pub fn queue_remove<D: Any + Send + Sync>(&mut self) {
        let tid = TypeId::of::<D>();
        self.data_to_remove.write().unwrap().push(tid);
    }

    /// Request that the engine loop should stop
    pub fn request_shutdown(&self) {
        self.should_run.store(false, Ordering::SeqCst);
    }
}
