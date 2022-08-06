//! Systems - separated engine modules running on dedicated threads

use super::{data::Data, DataTypeMap, SystemInterfaceTypeMap, TypeErasedData};
use std::{
    any::{Any, TypeId},
    sync::{
        atomic::{AtomicBool, Ordering},
        Barrier,
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
/// to finish whatever they were doing using a provided [`Barrier`]
pub trait System<'ctx>
where
    Self: Any + Send + Sync,
{
    /// Returns this system's name
    fn name(&self) -> &str;

    /// Called before the [`System::frame`] callback of this system, and only
    /// once, after it's added.
    fn init(&mut self, _context: &mut SystemContext<'ctx>) {}

    /// Runs the system through a frame, within its dedicated thread.
    /// Note, that the system has to wait for appropriate barriers, see
    /// [`SystemContext`] documentation for details.
    fn frame(&mut self, context: &mut SystemContext<'ctx>);
}

/// Some systems, for example the renderer, implement a public interface. This
/// trait can specify what kinda system interface this system implements.
pub trait HasSystemInterface: Any + Send + Sync {
    type SystemInterface: Any + Send + Sync;
    /// Creates the system interface. This function will only be called once
    /// on a single instance of this System. If it's ever called more than
    /// once, it's a bug.
    fn create_system_interface(&self) -> Self::SystemInterface;
}

/// Context passed to each system thread, which allows for communication with
/// the engine.
pub struct SystemContext<'ctx> {
    /// Specifies whether the game loop should run
    pub should_run: &'ctx AtomicBool,

    /// Barrier that synchronizes every frame's basic end, starting the post
    /// frame phase, allowing systems to do whatever late operations they need
    /// to do after primary processing.
    pub(super) frame_barrier: &'ctx Barrier,
    pub(super) frame_barrier_waited: bool,

    /// All system interfaces
    pub(super) system_interfaces: &'ctx SystemInterfaceTypeMap,

    /// All data available this frame
    pub(super) data: &'ctx DataTypeMap,

    /// Data types to be removed after this frame. Only one system can remove
    /// and add systems at a time, otherwise you'll get a desync
    pub(super) data_to_remove: &'ctx mut Vec<TypeId>,

    /// Data types to be added after this frame. See `data_to_remove` for sync
    /// warning
    pub(super) data_to_add: &'ctx mut Vec<(TypeId, TypeErasedData)>,

    /// All events caught this frame, with the exception of scale factor changes,
    /// since `WindowEvent<'static>` is unable to really store them
    pub events: &'ctx [WindowEvent<'static>],
}

impl<'ctx> SystemContext<'ctx> {
    /// Returns a reference to specified global data
    pub fn data<D: Data>(&self) -> &'ctx D::Storage {
        self.data
            .get(&TypeId::of::<D>())
            .expect("trying to access unregistered data")
            .downcast_ref::<D>()
            .expect("invalid data T->V mapping")
            .get_data()
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
    pub fn finish_frame(&mut self) {
        debug_assert!(!self.frame_barrier_waited, "Cannot finish frame twice");
        self.frame_barrier.wait();
        self.frame_barrier_waited = true;
    }

    /// Enqueues a specific data value to be added into the global engine list.
    /// The modification will happen next frame, and will overwrite any existing
    /// data.
    pub fn queue_add<D: Data>(&mut self, value: impl Into<Box<D>>) {
        let tid = TypeId::of::<D>();
        self.data_to_add.push((tid, value.into()));
    }

    /// Enqueues removal of specific data
    pub fn queue_remove<D: Data>(&mut self) {
        let tid = TypeId::of::<D>();
        self.data_to_remove.push(tid);
    }

    /// Request that the engine loop should stop
    pub fn request_shutdown(&self) {
        self.should_run.store(false, Ordering::SeqCst);
    }
}
