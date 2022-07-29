//! Systems, separated engine modules running on dedicated threads

use std::{sync::{Barrier, atomic::AtomicBool}, any::Any};
use super::data::Data;

/// Implements an engine system.
/// 
/// A system is a module which implements a certain piece of functionality,
/// like rendering, physics, or the overall entity/scene system. Each system
/// runs in a dedicated thread.
/// 
/// Systems perform their work each frame and then wait for other systems
/// to finish whatever they were doing using a provided [`Barrier`]
pub trait System: Any + Send + Sync {
    /// Returns this system's name
    fn name(&self) -> &str;
    /// Runs the system through another frame, within its dedicated thread.
    /// Note, that the system has to wait for appropriate barriers, see
    /// [`SystemContext`] documentation for details. 
    fn frame<'s>(&mut self, context: SystemContext<'s>);
}

/// Some systems, for example the renderer, implement a public interface. This
/// trait can specify what kinda system interface this system implements.
pub trait HasSystemInterface: System + Any + Sync {
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
/// Both barriers must be waited for.
#[derive(Clone)]
pub struct SystemContext<'e> {
    /// Barrier that synchronizes every frame's basic end, starting the post
    /// frame phase, allowing systems to do whatever late operations they need
    /// to do after primary processing.
    pub frame_barrier: &'e Barrier,

    /// Barrier that synchronizes the post frame phase. After this barrier
    /// resets, a new frame will begin to process.
    pub post_frame_barrier: &'e Barrier,

    pub(super) should_run: &'e AtomicBool,
}

impl<'e> SystemContext<'e> {
    pub fn data<T: Data>(&self) -> &'e T::Storage {
        todo!()
    }
}

