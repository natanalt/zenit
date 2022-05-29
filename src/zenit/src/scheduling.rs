use bevy_ecs::prelude::*;

#[derive(StageLabel, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EngineStage {
    Update,
}

pub fn create_schedule() -> Schedule {
    let mut schedule = Schedule::default();

    schedule
}
