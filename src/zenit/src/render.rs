use crevice::std140::AsStd140;
use glam::*;
use log::*;
use once_cell::sync::OnceCell;
use pollster::FutureExt;
use std::{cell::RefCell, iter, rc::Rc};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    *,
};
use winit::window::Window;
use zenit_utils::math::Radians;

pub struct Renderer {
    dc: DeviceContext,
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
            dc: DeviceContext {
                device,
                queue,
                instance,
                adapter,
                surface,
                sconfig,
                current_surface: None,
            },
            window,
        }
    }

    pub fn render_all(&mut self) {
        todo!()
    }
}

pub struct DeviceContext {
    pub device: Device,
    pub queue: Queue,
    pub instance: Instance,
    pub adapter: Adapter,
    pub surface: Surface,
    pub sconfig: SurfaceConfiguration,
    pub current_surface: Option<TextureView>,
}

pub struct Shader {
    pub name: String,
    pub module: ShaderModule,
    pub metadata: toml::Value,
}

#[macro_export]
macro_rules! include_shader {
    ($device:expr, $name:literal) => {
        $crate::render::Shader {
            name: String::from($name),
            module: ($device).create_shader_module(::wgpu::include_spirv!(concat!(
                env!("OUT_DIR"),
                "/",
                $name,
                ".spv"
            ))),
            metadata: include_str!(concat!(env!("OUT_DIR"), "/", $name, ".toml"))
                .parse::<toml::Value>()
                .unwrap(),
        }
    };
}

pub struct Camera {
    pub position: Vec3A,
    pub rotation: Quat,
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
            rotation: Quat::IDENTITY,
            fov: Radians::from_degrees(90.0),
            near_plane: 0.00001,
            far_plane: 10000.0,
            camera_buffer: device.create_buffer(&BufferDescriptor {
                label: Some("Camera"),
                size: CameraBuffer::std140_size_static() as u64,
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
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

        let forward = self.rotation * Vec3A::Z;
        let up = self.rotation * Vec3A::Y;
        let world_to_view = Mat4::look_at_lh(
            Vec3::from(self.position),
            Vec3::from(self.position + forward),
            Vec3::from(up),
        );

        let data = CameraBuffer {
            projection,
            world_to_view,
        };

        queue.write_buffer(&self.camera_buffer, 0, data.as_std140().as_bytes());
    }
}

/// A complete (and someday fairly cheaply cloneable) set of information
/// about everything that needs to be rendered.
/// 
/// During a frame's rendering, this is first locked by the process code, then
/// by the rendering code.
/// 
/// Once multithreading is brought back into the engine, this structure will
/// need to be copied cheaply, as it'll be copied by the render thread every
/// frame.
pub struct RenderState {

}

/// A scenario describes everything about a scene - the skybox, models, particles
/// and so on. A [`Viewport`] combines a scenario with a camera and render target,
/// allowing for a full render.
pub struct Scenario {
    pub skybox: Skybox,
    pub lights: Vec<Light>,
}

/// A skybox includes information about the skybox cubemap.
/// 
/// Someday it may also include information about directional light, or HDR info,
/// or who knows what, idfk yet lol
pub struct Skybox {}

pub enum Light {
    PointLight,
    Spotlight,
    DirectionalLight,
}

pub struct ModelInstance {
    pub model: Rc<Model>,
}

pub struct Model {

}
