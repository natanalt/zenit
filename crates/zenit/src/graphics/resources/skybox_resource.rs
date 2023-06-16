use crate::bind_group_layout_array;
use crate::graphics::DeviceContext;
use glam::*;
use parking_lot::Mutex;
use std::sync::Arc;
use wgpu::*;
use zenit_utils::ArcPoolHandle;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkyboxHandle(pub(in crate::graphics) ArcPoolHandle);

pub struct SkyboxResource {
    pub label: String,
    pub(in crate::graphics) gpu_resources: Arc<Mutex<SkyboxGpuResources>>,
}

impl SkyboxResource {
    pub fn new(_dc: &DeviceContext, _desc: &SkyboxDescriptor) -> Self {
        todo!()
    }
}

pub struct SkyboxGpuResources {
    pub bind_group: wgpu::BindGroup,
}

pub struct SkyboxRenderer {
    pub bind_group_layout: Arc<wgpu::BindGroupLayout>,
}

impl SkyboxRenderer {
    pub fn new(dc: &DeviceContext) -> Self {
        let bind_group_layout = Arc::new(dc.device.create_bind_group_layout(
            &BindGroupLayoutDescriptor {
                label: Some("Skybox Bind Group Layout"),
                entries: &bind_group_layout_array![
                    0 => (
                        FRAGMENT,
                        BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::Cube,
                            multisampled: false,
                        },
                    ),
                    1 => (
                        FRAGMENT,
                        BindingType::Sampler(SamplerBindingType::Filtering),
                    ),
                ],
            },
        ));

        Self { bind_group_layout }
    }

    pub fn render_skybox(
        &self,
        _dc: &DeviceContext,
        _skybox: &SkyboxGpuResources,
        _target: &TextureView,
    ) {
        todo!()
    }
}

pub struct SkyboxDescriptor {
    pub name: String,
    pub layer_dimension: UVec2,
}
