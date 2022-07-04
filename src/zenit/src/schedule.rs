use bevy_ecs::prelude::*;
use crate::render::scene::schedule::create_scene_scheduler;

#[derive(Debug, Clone, Hash, PartialEq, Eq, StageLabel)]
pub enum TopFrameStage {    
    /// Runs before FrameStart
    PreFrameStart,
    /// Begins a new frame by acquiring a new surface image, initializing
    /// control panel for this frame, etc. It also includes some initial
    /// widget placements for correct laoyout purposes.
    FrameStart,
    /// Does all scene rendering. Internally defined as another schedule,
    /// using [`crate::render::scene::schedule::SceneRenderStage`] as the label.
    Scene,
    /// All custom control panel widgets (like custom windows).
    /// They are executed in parallel, and access to the egui context is
    /// synchronized via a mutex lock. When this stage runs, egui context
    /// is set up for the current frame.
    ///
    /// Scenes are processed internally by widgets, likely through their own
    /// dedicated worlds and schedules.
    ControlPanel,
    /// Finishes this frame. Submits command buffers to the GPU, finalizes
    /// egui rendering, and so on.
    FrameFinish,
    /// Runs after FrameFinish
    PostFrameFinish,
}

pub fn create_top_scheduler() -> Schedule {
    let mut s = Schedule::default();
    s.add_stage(TopFrameStage::PreFrameStart, SystemStage::parallel());
    s.add_stage(TopFrameStage::FrameStart, SystemStage::parallel());
    s.add_stage(TopFrameStage::Scene, create_scene_scheduler());
    s.add_stage(TopFrameStage::ControlPanel, SystemStage::parallel());
    s.add_stage(TopFrameStage::FrameFinish, SystemStage::parallel());
    s.add_stage(TopFrameStage::PostFrameFinish, SystemStage::parallel());
    s
}
