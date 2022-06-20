use bevy_ecs::prelude::*;

#[derive(Debug, Clone, Hash, PartialEq, Eq, StageLabel)]
pub enum TopFrameStage {
    /// Begins a new frame by acquiring a new swapchain image and generally
    /// setting up application state for rendering.
    RenderStart,
    /// Base control panel widgets display that need to be executed sequentially
    /// to ensure correct layout (basically just root top and side panels).
    /// Also includes any necessary egui per-frame initialization.
    CtPanelStart,
    /// All custom control panel widgets (like custom windows).
    /// They are executed in parallel, and access to the egui context is
    /// synchronized via a mutex lock.
    ///
    /// Scenes are processed internally by widgets, likely through their own
    /// dedicated worlds and schedules.
    CtPanelWidgets,
    /// Finishes control panel rendering
    CtPanelFinish,
    /// Submits all command buffers to the GPU and finishes this frame on the
    /// renderer side.
    RenderFinish,
}

pub fn create_top_scheduler() -> Schedule {
    let mut s = Schedule::default();
    s.add_stage(TopFrameStage::RenderStart, SystemStage::single_threaded());
    s.add_stage(TopFrameStage::CtPanelStart, SystemStage::single_threaded());
    s.add_stage(TopFrameStage::CtPanelWidgets, SystemStage::parallel());
    s.add_stage(TopFrameStage::CtPanelFinish, SystemStage::single_threaded());
    s.add_stage(TopFrameStage::RenderFinish, SystemStage::single_threaded());
    s
}
