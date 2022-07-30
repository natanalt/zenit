//! Systems, separated engine modules running on dedicated threads

use super::{data::Data, DataTypeMap, SystemInterfaceTypeMap, TypeErasedData};
use std::{
    any::{Any, TypeId},
    sync::{atomic::AtomicBool, Barrier},
};

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
    fn init(&mut self, _context: &SystemContext<'ctx>) {}

    /// Runs the system through a frame, within its dedicated thread.
    /// Note, that the system has to wait for appropriate barriers, see
    /// [`SystemContext`] documentation for details.
    fn frame(&mut self, context: &SystemContext<'ctx>);
}

/// Some systems, for example the renderer, implement a public interface. This
/// trait can specify what kinda system interface this system implements.
pub trait HasSystemInterface: Any + Sync {
    type SystemInterface: Any + Sync;
    /// Creates the system interface. This function will only be called once
    /// on a single instance of this System. If it's ever called more than
    /// once, it's a bug.
    fn create_system_interface(&self) -> Self::SystemInterface;
}

/// Context passed to each system thread, which allows for communication with
/// the engine.
///
/// ## Sync barriers
/// The context includes 2 barriers: *frame barrier* and *post frame barrier*.
/// They are meant to be awaited in a very specific order:
/// ```text
///               ↓ frame_barrier    ↓ post_frame_barrier
/// ┌─────────────┬──────────────────┐
/// │ Frame stuff │ Post frame stuff │
/// └─────────────┴──────────────────┘
/// ↑ frame start                    ↑ frame end
/// ```
/// Both barriers must be waited for during the system's frame callback. They
/// must be awaited for in the specific order (frame -> post-frame), otherwise
/// you'll get a deadlock.
pub struct SystemContext<'ctx> {
    /// Barrier that synchronizes every frame's basic end, starting the post
    /// frame phase, allowing systems to do whatever late operations they need
    /// to do after primary processing.
    pub frame_barrier: &'ctx Barrier,

    /// Barrier that synchronizes the post frame phase. After this barrier
    /// resets, a new frame will begin to process.
    pub post_frame_barrier: &'ctx Barrier,

    /// Specifies whether the game loop should run
    pub should_run: &'ctx AtomicBool,

    /// All system interfaces
    pub system_interfaces: &'ctx SystemInterfaceTypeMap,

    /// All data available this frame
    pub data: &'ctx DataTypeMap,

    /// Data types to be removed after this frame. Only one system can remove
    /// and add systems at a time, otherwise you'll get a desync
    pub data_to_remove: &'ctx mut Vec<TypeId>,

    /// Data types to be added after this frame. See `data_to_remove` for sync
    /// warning
    pub data_to_add: &'ctx mut Vec<(TypeId, TypeErasedData)>,
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
}
