use super::{
    system::{HasSystemInterface, System},
    DataTypeMap, Engine, SystemInterfaceTypeMap,
};
use std::{
    any::{Any, TypeId},
    sync::{
        atomic::AtomicBool,
        mpsc::{self, Sender},
        Arc,
    },
    thread,
};
use winit::event::WindowEvent;

/// It builds the engine. Very surprising, I know
#[derive(Default)]
pub struct EngineBuilder {
    systems: Vec<Box<dyn for<'a> System<'a>>>,
    system_interfaces: SystemInterfaceTypeMap,
    data: DataTypeMap,
}

impl EngineBuilder {
    /// Creates and includes a [`System`] instance, if it implements [`Default`]
    pub fn make_system<S>(mut self) -> Self
    where
        S: for<'a> System<'a> + HasSystemInterface + Default,
    {
        let system = Box::new(S::default());
        let system_interface = Box::new(system.create_system_interface());
        self.systems.push(system);
        self.system_interfaces
            .insert(TypeId::of::<S::SystemInterface>(), system_interface);
        self
    }

    /// Includes specified data
    pub fn with_data<D>(mut self, data: D) -> Self
    where
        D: Any + Send + Sync,
    {
        self.data.insert(TypeId::of::<D>(), Box::new(data));
        self
    }

    /// Creates and includes a data instance, if it implements [`Default`]
    pub fn make_data<D>(self) -> Self
    where
        D: Any + Send + Sync + Default,
    {
        self.with_data(D::default())
    }

    /// Finalizes the engine build and starts it in a separate thread.
    pub fn build_and_run(self) -> EngineCommunication {
        let (event_sender, event_receiver) = mpsc::channel();
        let should_run = Arc::new(AtomicBool::new(true));
        let is_running = Arc::new(AtomicBool::new(false));

        let engine = Engine {
            systems: self.systems,
            system_interfaces: self.system_interfaces,
            data: self.data,
            event_receiver,
            should_run: should_run.clone(),
            is_running: is_running.clone(),
        };

        let comms = EngineCommunication {
            event_sender,
            should_run,
            is_running,
        };

        thread::Builder::new()
            .name("Engine Controller".to_string())
            .spawn(move || engine.run())
            .expect("couldn't spawn main engine thread");

        comms
    }
}

/// Communication between the main engine control thread and the main GUI event
/// loop thread. Basically it's the way the GUI thread can:
///  * Report new window and system events to the engine.
///  * Report window closing done by the user, requiring immediate shutdown.
///    Note, that when the window close button is pressed, the window is closed
///    instantly.
pub struct EngineCommunication {
    /// Send channel for event loop's events
    pub event_sender: Sender<WindowEvent<'static>>,
    /// If true, next iteration of the game loop will process a new frame
    pub should_run: Arc<AtomicBool>,
    /// If true, the engine is still running.
    pub is_running: Arc<AtomicBool>,
}
