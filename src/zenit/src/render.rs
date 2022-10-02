use crevice::std140::AsStd140;
use glam::*;
use log::*;
use pollster::FutureExt;
use std::{borrow::Borrow, mem, rc::Rc};
use wgpu::*;
use winit::window::Window;
use zenit_utils::math::Radians;

pub struct Renderer {
    instance: Instance,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    surface: Surface,
    sconfig: SurfaceConfiguration,
    window: Rc<Window>,
}

impl Renderer {
    pub fn new(window: Rc<Window>) -> Self {
        let instance = Instance::new(Backends::all());
        let surface = unsafe { instance.create_surface(&window) };

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .block_on()
            .expect("couldn't find a GPU");

        info!("Using adapter: {}", adapter.get_info().name);
        info!("Using backend: {:?}", adapter.get_info().backend);

        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    label: None,
                    // BC compression, aka DXTn or S3
                    features: Features::TEXTURE_COMPRESSION_BC,
                    limits: Limits::default(),
                },
                None,
            )
            .block_on()
            .expect("couldn't create a device");

        let sconfig = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: window.inner_size().width,
            height: window.inner_size().height,
            present_mode: PresentMode::AutoVsync,
        };

        surface.configure(&device, &sconfig);

        Self {
            instance,
            adapter,
            device,
            queue,
            surface,
            sconfig,
            window,
        }
    }

    pub fn render_all(&mut self) {}
}

pub struct Camera {
    pub position: Vec3A,
    /// Rotation represented by Euler angles, in radians
    /// X represents yaw, Y represents pitch, Z represents roll
    pub rotation: Vec3A,
    pub fov: Radians,
    pub near_plane: f32,
    pub far_plane: f32,

    /// Update with [`update_buffer`]
    pub camera_buffer: Buffer,
}

#[derive(AsStd140)]
struct CameraBuffer {
    projection: Mat4,
    world_to_view: Mat4,
}

impl Camera {
    pub fn new(device: &Device) -> Self {
        Self {
            position: Vec3A::ZERO,
            rotation: Vec3A::ZERO,
            fov: Radians::from_degrees(90.0),
            near_plane: 0.00001,
            far_plane: 10000.0,
            camera_buffer: device.create_buffer(&BufferDescriptor {
                label: Some("Camera"),
                size: CameraBuffer::std140_size_static() as u64,
                usage: BufferUsages::UNIFORM,
                mapped_at_creation: false,
            }),
        }
    }

    /// Updates the internal uniform buffer. This is done every frame by the main renderer code
    /// automatically for all used cameras.
    pub fn update_buffer(&self, aspect_ratio: f32, queue: &Queue) {
        let projection = Mat4::perspective_lh(
            self.fov.to_radians(),
            aspect_ratio,
            self.near_plane,
            self.far_plane,
        );

        // TODO: rotations
        let world_to_view = Mat4::from_translation((-self.position).into());

        let data = CameraBuffer {
            projection,
            world_to_view,
        };

        queue.write_buffer(&self.camera_buffer, 0, data.as_std140().as_bytes());
    }
}

pub struct Viewport {
    pub target: Rc<TextureView>,
}
