use crate::render::DeviceContext;
use crevice::std140::AsStd140;
use glam::*;
use parking_lot::{Mutex, MutexGuard};
use std::f32::consts::FRAC_PI_2;
use wgpu::*;
use zenit_utils::math::*;

pub struct Camera {
    pub info: Mutex<CameraInfo>,
    pub renderer_info: Mutex<RendererCameraInfo>,
}

impl Camera {
    #[inline]
    pub fn lock_info(&self) -> MutexGuard<CameraInfo> {
        self.info.lock()
    }
}

/// Any camera info used by the renderer. Scene code should not be bothered by it.
pub struct RendererCameraInfo {
    /// [`CameraInfo`] instance from the last frame, saved by the renderer in post process.
    pub pending_info: CameraInfo,
    /// Camera's uniform buffer, updated each frame the camera is used.
    pub buffer: wgpu::Buffer,
    /// The actual render texture.
    pub target: wgpu::Texture,
}

impl RendererCameraInfo {
    pub fn new(dc: &DeviceContext) -> Self {
        Self {
            pending_info: CameraInfo::default(),
            buffer: dc.device.create_buffer(&BufferDescriptor {
                label: Some("Camera uniform buffer"),
                size: CameraBuffer::std140_size_static() as u64,
                usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
                mapped_at_creation: false,
            }),
            target: Self::create_texture(&CameraInfo::default(), dc),
        }
    }

    /// Schedules a resource update:
    ///  * the camera uniform buffer is sent update data
    ///  * render texture is recreated if its size changed
    pub fn update_resources(&mut self, dc: &DeviceContext) {
        // Always reupload camera uniform buffer data, it's fair to assume that it changes a lot
        let data = CameraBuffer::from_camera(&self.pending_info);
        let formatted = data.as_std140();
        dc.queue.write_buffer(&self.buffer, 0, formatted.as_bytes());

        let width_matches = self.target.width() == self.pending_info.texture_width;
        let height_matches = self.target.height() == self.pending_info.texture_height;
        if !width_matches || !height_matches {
            self.target = Self::create_texture(&self.pending_info, dc);
        }
    }

    fn create_texture(ci: &CameraInfo, dc: &DeviceContext) -> wgpu::Texture {
        dc.device.create_texture(&TextureDescriptor {
            label: Some("Camera target"),
            size: Extent3d {
                width: ci.texture_width,
                height: ci.texture_height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1, // TODO: MSAA
            dimension: TextureDimension::D2,
            format: crate::render::RENDER_FORMAT,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[crate::render::RENDER_FORMAT],
        })
    }
}

/// Represents the public information of a [`Camera`]. This information is uploaded to the camera's
/// uniform buffer every frame, if it's used.
#[derive(Debug, Clone, PartialEq)]
pub struct CameraInfo {
    pub position: Vec3A,
    pub rotation: Quat,
    pub fov: Radians,
    pub near_plane: f32,
    pub far_plane: f32,
    pub texture_width: u32,
    pub texture_height: u32,
}

impl Default for CameraInfo {
    fn default() -> Self {
        Self::new()
    }
}

impl CameraInfo {
    /// Creates a new [`CameraInfo`] object with the following properties:
    ///  * position at origin,
    ///  * no rotation (identity quaternion)
    ///  * 90° field of view
    ///  * 0.00001 .. 10000.0 view distance (near/far planes)
    ///  * 1024x768 render texture dimensions
    pub const fn new() -> Self {
        Self {
            position: Vec3A::ZERO,
            rotation: Quat::IDENTITY,
            fov: Radians(FRAC_PI_2), // 90 degrees
            near_plane: 0.00001,
            far_plane: 10000.0,
            texture_width: 1024,
            texture_height: 768,
        }
    }
}

/// Camera buffer, used in shader uniform buffers.
///
/// Keep this uniform buffer in sync with `shaders/camera.inc.wgsl`
#[derive(AsStd140)]
pub struct CameraBuffer {
    pub projection: Mat4,
    pub world_to_view: Mat4,
    pub texture_size: UVec2,
}

impl CameraBuffer {
    #[inline]
    pub fn from_camera(camera: &CameraInfo) -> Self {
        let (width, height) = (camera.texture_width as f32, camera.texture_height as f32);
        let aspect_ratio = width / height;

        let projection = Mat4::perspective_lh(
            camera.fov.to_radians(),
            aspect_ratio,
            camera.near_plane,
            camera.far_plane,
        );

        let forward = camera.rotation * Vec3A::Z;
        let up = camera.rotation * Vec3A::Y;
        let world_to_view = Mat4::look_at_lh(
            Vec3::from(camera.position),
            Vec3::from(camera.position + forward),
            Vec3::from(up),
        );

        let texture_size = uvec2(camera.texture_width, camera.texture_height);

        Self {
            projection,
            world_to_view,
            texture_size,
        }
    }
}
