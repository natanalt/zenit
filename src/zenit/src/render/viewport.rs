use derive_builder::Builder;
use glam::IVec2;

pub struct Viewport;

#[derive(Builder)]
pub struct ViewportCreationInfo {
    pub resolution: IVec2,
}
