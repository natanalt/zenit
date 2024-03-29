use crate::bind_group_layout_array;
use crate::entities::{Component, Entity};
use crate::graphics::{DeviceContext, Renderer};
use crevice::std140::AsStd140;
use glam::*;
use parking_lot::Mutex;
use std::num::NonZeroU64;
use std::sync::Arc;
use wgpu::*;
use zenit_utils::{
    math::{AngleExt, Radians},
    ArcPoolHandle,
};

/// Handle to a [`CameraResource`]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CameraHandle(pub(in crate::graphics) ArcPoolHandle);

/// A camera contains information about how a camera should be rendered, including its target
/// texture.
#[derive(Clone)]
pub struct CameraResource {
    pub fov: Radians,
    pub near_plane: f32,
    pub far_plane: f32,
    pub texture_size: UVec2,

    pub(in crate::graphics) gpu_resources: Arc<Mutex<CameraGpuResources>>,
}

impl CameraResource {
    pub fn new(r: &mut Renderer, desc: &CameraDescriptor) -> Self {
        Self {
            fov: desc.fov,
            near_plane: desc.near_plane,
            far_plane: desc.far_plane,
            texture_size: desc.texture_size,
            gpu_resources: Arc::new(Mutex::new(CameraGpuResources::new(r, desc))),
        }
    }

    pub fn resize(&mut self, dc: &DeviceContext, new_size: UVec2) {
        self.texture_size = new_size;
        self.gpu_resources.lock().recreate_texture(dc, new_size);
    }
}

pub struct CameraGpuResources {
    pub texture: wgpu::Texture,
    pub texture_view: wgpu::TextureView,
    pub depth_texture: wgpu::Texture,
    pub depth_texture_view: wgpu::TextureView,
    pub buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl CameraGpuResources {
    pub fn register_camera_bind_layout(r: &mut Renderer) {
        r.shared_bind_layouts.insert("Camera".to_string(), r.dc.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("camera bind group layout"),
            entries: &bind_group_layout_array![
                0 => (
                    VERTEX | FRAGMENT,
                    BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: NonZeroU64::new(CameraBuffer::std140_size_static() as u64),
                    }
                )
            ],
        }));
    }

    pub fn new(r: &mut Renderer, desc: &CameraDescriptor) -> Self {
        let (texture, texture_view) = create_camera_texture(&r.dc, desc.texture_size);
        let (depth_texture, depth_texture_view) =
            create_camera_depth_texture(&r.dc, desc.texture_size);

        let buffer = r.dc.device.create_buffer(&BufferDescriptor {
            label: Some("Camera uniform buffer"),
            size: CameraBuffer::std140_size_static() as u64,
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });

        Self {
            texture,
            texture_view,
            depth_texture,
            depth_texture_view,
            bind_group: r.dc.device.create_bind_group(&BindGroupDescriptor {
                label: Some("camera bind group"),
                layout: &r
                    .shared_bind_layouts
                    .get("Camera")
                    .expect("camera bind group layout missing"),
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::Buffer(BufferBinding {
                        buffer: &buffer,
                        offset: 0,
                        size: None,
                    }),
                }],
            }),
            buffer,
        }
    }

    pub fn update_camera_buffer(&self, dc: &DeviceContext, buffer: &CameraBuffer) {
        let data = buffer.as_std140();
        dc.queue.write_buffer(&self.buffer, 0, data.as_bytes());
    }

    pub fn recreate_texture(&mut self, dc: &DeviceContext, size: UVec2) {
        let (texture, view) = create_camera_texture(dc, size);
        self.texture = texture;
        self.texture_view = view;
        let (texture, view) = create_camera_depth_texture(dc, size);
        self.depth_texture = texture;
        self.depth_texture_view = view;
    }
}

fn create_camera_texture(dc: &DeviceContext, size: UVec2) -> (wgpu::Texture, wgpu::TextureView) {
    let desc = TextureDescriptor {
        label: Some("Camera color target"),
        size: Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1, // TODO: MSAA
        dimension: TextureDimension::D2,
        format: crate::graphics::RENDER_FORMAT,
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    };

    let texture = dc.device.create_texture(&desc);

    let view = texture.create_view(&TextureViewDescriptor {
        label: Some("Camera target view"),
        format: Some(desc.format),
        dimension: Some(TextureViewDimension::D2),
        aspect: TextureAspect::All,
        base_mip_level: 0,
        mip_level_count: None,
        base_array_layer: 0,
        array_layer_count: None,
    });

    (texture, view)
}

fn create_camera_depth_texture(
    dc: &DeviceContext,
    size: UVec2,
) -> (wgpu::Texture, wgpu::TextureView) {
    let desc = TextureDescriptor {
        label: Some("Camera depth target"),
        size: Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: crate::graphics::DEPTH_FORMAT,
        usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    };

    let texture = dc.device.create_texture(&desc);

    let view = texture.create_view(&TextureViewDescriptor {
        label: Some("Camera target view"),
        format: Some(desc.format),
        dimension: Some(TextureViewDimension::D2),
        aspect: TextureAspect::All,
        base_mip_level: 0,
        mip_level_count: None,
        base_array_layer: 0,
        array_layer_count: None,
    });

    (texture, view)
}

/// Camera buffer, used in shader uniform buffers.
///
/// Keep this uniform buffer in sync with `shaders/shared/camera.inc.wgsl`
#[derive(AsStd140)]
pub struct CameraBuffer {
    pub projection: Mat4,
    pub world_to_view: Mat4,
    pub texture_size: UVec2,
}

impl CameraBuffer {
    pub fn new(camera: &CameraResource, transform: &Affine3A) -> Self {
        let texture_size = camera.texture_size;

        let width = texture_size.x as f32;
        let height = texture_size.y as f32;
        let aspect_ratio = width / height;

        let projection = Mat4::perspective_rh(
            camera.fov.to_radians(),
            aspect_ratio,
            camera.near_plane,
            camera.far_plane,
        );

        let position = transform.translation;
        let rotation = Quat::from_affine3(transform);

        // Not sure how good of a way this is lol
        let forward = rotation * Vec3A::Z;
        let up = rotation * Vec3A::Y;
        let world_to_view = Mat4::look_at_rh(
            Vec3::from(position),
            Vec3::from(position + forward),
            Vec3::from(up),
        );

        Self {
            projection,
            world_to_view,
            texture_size,
        }
    }
}

// TODO: what happens if multiple CameraComponents link to the same camera resource?

pub struct CameraComponent {
    /// If true, this camera will be rendered to.
    pub enabled: bool,
    /// Underlying camera resource to render to
    pub camera_handle: CameraHandle,
    /// Target entity with a [`crate::graphics::SceneComponent`].
    pub scene_entity: Entity,
}

impl Component for CameraComponent {}

pub struct CameraDescriptor {
    pub texture_size: UVec2,
    pub fov: Radians,
    pub near_plane: f32,
    pub far_plane: f32,
}

impl Default for CameraDescriptor {
    fn default() -> Self {
        Self {
            texture_size: uvec2(640, 480),
            fov: 90f32.radians(),
            near_plane: 0.0001,
            far_plane: 1000.0,
        }
    }
}
