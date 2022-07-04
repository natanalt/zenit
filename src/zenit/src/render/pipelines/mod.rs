use std::{
    any::{Any, TypeId},
    collections::HashMap,
};
use wgpu::TextureFormat;
use super::RenderContext;

pub mod camera;
pub mod skybox;

pub trait PipelineSource: Any + Send + Sync {
    fn new(ctx: &RenderContext, format: TextureFormat) -> Self;
}

#[derive(Default)]
pub struct PipelineStorage {
    pub pipelines: HashMap<(TypeId, TextureFormat), Box<dyn Any + Send + Sync>>,
}

impl PipelineStorage {
    /// Returns an existing pipeline source T for given texture format, or
    /// initializes it using provided [`RenderContext`].
    pub fn get<T>(&mut self, ctx: &RenderContext, format: TextureFormat) -> &mut T
    where
        T: PipelineSource,
    {
        self.pipelines
            .entry((TypeId::of::<T>(), format))
            .or_insert_with(|| Box::new(T::new(ctx, format)))
            .downcast_mut()
            .unwrap()
    }
}
