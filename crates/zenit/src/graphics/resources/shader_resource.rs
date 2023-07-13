use crate::graphics::Renderer;
use std::borrow::Cow;
use wgpu::*;

/// A shader module loaded by [`wgpu`], available for use in pipelines.
pub struct ShaderResource {
    pub name: String,
    pub code: String,
    pub module: ShaderModule,
}

impl ShaderResource {
    pub fn new(r: &mut Renderer, desc: &ShaderDescriptor) -> Self {
        let ShaderDescriptor { name, code } = desc.clone();

        let module = r.dc.device.create_shader_module(ShaderModuleDescriptor {
            label: Some(&name),
            source: ShaderSource::Wgsl(Cow::Borrowed(&code)),
        });

        Self { name, code, module }
    }
}

#[derive(Clone)]
pub struct ShaderDescriptor {
    pub name: String,
    pub code: String,
}
