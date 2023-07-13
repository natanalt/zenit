//! The core engine scheduling
//!
//! Zenit Engine tries to do basic multithreading with the top level engine modules. This allows
//! the main processing code, rendering code, event loop, etc. to be executed in parallel.
//!
//! Each one of those top level modules implements a trait called [`System`], and is futher referred
//! to as a system. They are synchronized and managed by the Engine Controller thread. During each
//! frame, processing of all systems is split into the following stages:
//!  * Initialization
//!  * Main Processing
//!  * Post Processing
//!
//! Each of these stages is a barrier, so for example, post processing only begins once all systems
//! have finished their main processing.
//!
//! ## Notes for implementing systems
//!  * Try to not assume that each system executes in parallel. While this is the case now, in the
//!    future a singlethreaded mode may be implemented, in which systems are executed sequentially
//!    for each phase.
//!

use ahash::AHashMap;
use log::*;
use parking_lot::{Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::{
    any::{Any, TypeId},
    iter,
    sync::atomic::{AtomicBool, Ordering},
    thread,
};
use winit::event::WindowEvent;

#[doc(inline)]
pub use bus::*;
mod bus;

#[doc(inline)]
pub use runner::*;
mod runner;

#[doc(inline)]
pub use frame_profiler::*;
mod frame_profiler;

/// Global immutable engine data.
pub struct EngineContext {
    /// Window event list modified by the event loop and the engine controller.
    ///
    /// Every frame this vector is sent to the event bus.
    pub window_events: Mutex<Vec<WindowEvent<'static>>>,
    /// See: [`Self::request_shutdown`]
    should_run: AtomicBool,
    /// Flag which determines whether the engine controller is still running.
    ///
    /// This should only be modified by the controller thread. Other code can freely load this value,
    /// though.
    pub is_running: AtomicBool,
    /// See: [`GlobalState`]
    pub globals: RwLock<GlobalState>,
}

impl Default for EngineContext {
    fn default() -> Self {
        Self {
            window_events: Mutex::new(Vec::new()),
            should_run: AtomicBool::new(true),
            is_running: AtomicBool::new(true),
            globals: RwLock::new(GlobalState::default()),
        }
    }
}

impl EngineContext {
    /// Tells the engine controller to break out of its loop at the end of the current frame.
    pub fn request_shutdown(&self) {
        trace!(
            "Thread `{}` requesting shutdown",
            thread::current().name().unwrap_or("(no name)")
        );
        self.should_run.store(false, Ordering::Release);
    }
}

/// The [`GlobalState`] contains data that may require mutability, or isn't related to
/// the engine's core execution.
///
/// Available in the [`EngineContext`] as a read-write locked resource. During main processing,
/// it is provided as an immutable reference, but systems are allowed to briefly write lock it in
/// other stages of execution.
#[derive(Default)]
pub struct GlobalState {
    pub data: AHashMap<TypeId, Box<dyn Any + Send + Sync>>,
}

impl GlobalState {
    // TODO: improve error messages for lock/read/write (only if debug_assertions)

    pub fn add_any(&mut self, t: impl Any + Send + Sync) {
        self.data.insert(t.type_id(), Box::new(t));
    }

    pub fn exists<T: Any + Send + Sync>(&self) -> bool {
        self.data.contains_key(&TypeId::of::<T>())
    }

    pub fn get<T: Any + Send + Sync>(&self) -> &T {
        self.data
            .get(&TypeId::of::<T>())
            .expect("global resource not found")
            .downcast_ref::<T>()
            .expect("corrupted global state mapping")
    }

    pub fn get_mut<T: Any + Send + Sync>(&mut self) -> &mut T {
        self.data
            .get_mut(&TypeId::of::<T>())
            .expect("global resource not found")
            .downcast_mut::<T>()
            .expect("corrupted global state mapping")
    }

    pub fn add_lockable<T: Any + Send>(&mut self, t: T) {
        self.data
            .insert(TypeId::of::<Mutex<T>>(), Box::new(Mutex::new(t)));
    }

    pub fn lock<T: Any + Send>(&self) -> MutexGuard<T> {
        self.data
            .get(&TypeId::of::<Mutex<T>>())
            .expect("lockable global resource not found")
            .downcast_ref::<Mutex<T>>()
            .expect("corrupted global state type mapping")
            .lock()
    }

    pub fn add_rw_lockable<T: Any + Send + Sync>(&mut self, t: T) {
        self.data
            .insert(TypeId::of::<RwLock<T>>(), Box::new(RwLock::new(t)));
    }

    pub fn read<T: Any + Send + Sync>(&self) -> RwLockReadGuard<T> {
        self.data
            .get(&TypeId::of::<RwLock<T>>())
            .expect("rw-lockable global resource not found")
            .downcast_ref::<RwLock<T>>()
            .expect("corrupted global state type mapping")
            .read()
    }

    pub fn write<T: Any + Send + Sync>(&self) -> RwLockWriteGuard<T> {
        self.data
            .get(&TypeId::of::<RwLock<T>>())
            .expect("rw-lockable global resource not found")
            .downcast_ref::<RwLock<T>>()
            .expect("corrupted global state type mapping")
            .write()
    }

    /// Enqueues a bus message for the next frame.
    pub fn send_message<T>(&self, message: T)
    where
        T: Any + Send + Sync + Into<Box<T>>,
    {
        let boxed: Box<dyn Any + Send + Sync> = message.into();

        self.get::<EngineBus>().send_messages(iter::once(boxed));
    }

    pub fn send_messages<T>(&self, messages: impl IntoIterator<Item = T>)
    where
        T: Any + Send + Sync + Into<Box<T>>,
    {
        self.get::<EngineBus>()
            .send_messages(messages.into_iter().map(|event| event.into() as _))
    }

    /// Creates an iterator over all messages sent this frame.
    pub fn new_messages(&self) -> impl Iterator<Item = &(dyn Any + Send + Sync)> {
        self.get::<EngineBus>().iter_messages()
    }

    /// Creates an iterator that only goes over messages of a specified type.
    pub fn new_messages_of<T>(&self) -> impl Iterator<Item = &T>
    where
        T: Any + Send + Sync,
    {
        self.get::<EngineBus>()
            .iter_messages()
            .filter_map(|msg| msg.downcast_ref())
    }
}

/// Trait implemented by engine systems, see module docs for details.
pub trait System: Any + Send {
    /// The system's label, primarily used to identify individual worker threads.
    /// It must be a constant.
    fn label(&self) -> &'static str;

    /// An initialization function called exactly once for each system.
    ///
    /// It is called **inside the engine thread**, sequentially for each system, in order of
    /// registration. This is the only time a system may also take a mutable reference to
    /// the [`EngineContext`].
    fn init(&mut self, ec: &mut EngineContext) {
        let _ = ec;
    }

    /// Called before the first frame initialization stage. This is different from [`System::init`]
    /// in that it's actually called during a frame's processing.
    fn first_frame(&mut self, ec: &EngineContext) {
        let _ = ec;
    }

    /// System frame stage 1.
    fn frame_initialization(&mut self, ec: &EngineContext) {
        let _ = ec;
    }

    /// System frame stage 2.
    fn main_process(&mut self, ec: &EngineContext, gc: &GlobalState) {
        let _ = (ec, gc);
    }

    /// System frame stage 3.
    fn post_process(&mut self, ec: &EngineContext) {
        let _ = ec;
    }
}
