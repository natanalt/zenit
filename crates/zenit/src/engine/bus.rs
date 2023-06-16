use parking_lot::FairMutex;
use std::{any::Any, mem};

type MessageVec = Vec<Box<dyn Any + Send + Sync>>;

/// The [`EngineBus`] implements a bus, a dedicated message queue for all systems.
///
/// To be more exact, any system can enqueue a message in the bus, and it'll become available for
/// read by all systems next frame.
/// The bus is used for sending short frame-long information, and can be used for inter-system
/// communication. For example, new window events are routed through it.
///
/// The [`EngineBus`] structure allows low level access to the message sending system, and you
/// should consider using functions stored in [`crate::engine::GlobalState`] for more
/// accessible functionality
pub struct EngineBus {
    /// Messages scheduled for the current frame.
    pub current_messages: MessageVec,
    /// Messages secheduled for the next frame.
    pub queued_messages: FairMutex<MessageVec>,
}

impl EngineBus {
    pub fn new() -> Self {
        Self {
            current_messages: MessageVec::with_capacity(512),
            queued_messages: FairMutex::new(MessageVec::with_capacity(512)),
        }
    }

    pub fn send_messages(&self, messages: impl Iterator<Item = Box<dyn Any + Send + Sync>>) {
        self.queued_messages.lock().extend(messages);
    }

    /// Moves queued messages into the current message queue
    pub fn next_frame(&mut self) {
        self.current_messages = mem::take(self.queued_messages.get_mut());
    }

    /// Iterates over messages sent this frame
    pub fn iter_messages(&self) -> impl Iterator<Item = &(dyn Any + Send + Sync)> {
        self.current_messages.iter().map(Box::as_ref)
    }
}
