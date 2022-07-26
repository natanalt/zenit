//! # The ECS scene system
//! This module contains everything related to scenes, entities, components
//! and all the fanciness. This is what puts the engine into motion and talks
//! with other systems to do ✨ stuff ✨

use crate::engine::{System, SystemContext};
use std::{
    sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}},
    thread::{self, JoinHandle},
};

#[derive(Default)]
pub struct SceneSystem {
    thread_handle: Mutex<Option<JoinHandle<()>>>,
    should_run: Arc<AtomicBool>,
}

impl System for SceneSystem {
    fn name(&self) -> &str {
        "Scene System"
    }

    fn start_thread(&self, context: Arc<SystemContext>) {
        let mut thread_handle = self.thread_handle.lock().unwrap();
        assert!(thread_handle.is_none(), "scene system is already running");

        let should_run = self.should_run.clone();
        self.should_run.store(true, Ordering::SeqCst);
        *thread_handle = Some(
            thread::Builder::new()
                .name(self.name().to_string())
                .spawn(move || scene_thread(context, should_run))
                .expect("couldn't spawn scene system thread"),
        );
    }

    fn request_stop(&self) {
        self.should_run.store(false, Ordering::SeqCst);
    }

    fn wait_for_stop(&self) {
        let mut thread_handle = self.thread_handle.lock().unwrap();
        if let Some(thread) = thread_handle.take() {
            thread.join().unwrap();
        }
    }

    fn is_running(&self) -> bool {
        self.thread_handle.lock().unwrap().is_some()
    }
}

fn scene_thread(context: Arc<SystemContext>, should_run: Arc<AtomicBool>) {
    while should_run.load(Ordering::SeqCst) {
        context.frame_barrier.wait();
        context.post_frame_barrier.wait();
    }
}
