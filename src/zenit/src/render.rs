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

#[allow(dead_code)]
pub struct Renderer {
    instance: Instance,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    surface: Surface,
    sconfig: SurfaceConfiguration,
    window: Rc<Window>,
    viewports: Vec<Rc<RefCell<Viewport>>>,
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
            viewports: vec![Rc::new(RefCell::new(Viewport {
                size: uvec2(sconfig.width, sconfig.height),
                color_target: Rc::new(todo!()),
                camera: Camera::new(&device),
                clear_color: Some(vec4(1.0, 0.0, 1.0, 1.0)),
                scenario: Rc::new(RefCell::new(Scenario {
                    instances: vec![Rc::new(RefCell::new(ModelInstance {
                        model: Rc::new(Model {
                            name: "Triangle".to_string(),
                            vertex_buffer: device.create_buffer_init(&BufferInitDescriptor {
                                label: Some("Triangle"),
                                contents: &[
                                    0x00,
                                    0x00,
                                    0x00,
                                    0x00,
                                    0x00,
                                    0x00,
                                    0x00,
                                    0x3F,
                                    0x00,
                                    0x00,
                                    0x80,
                                    0x3F,
                                    0x00,
                                    0x00,
                                    0x00,
                                    0x00,
                                    0x00,
                                    0x00,
                                    0x00,
                                    0x00,
                                    0x00,
                                    0x00,
                                    0x80,
                                    0x3F,
                                    0x00,
                                    0x00,
                                    0x00,
                                    0x3F,
                                    0x00,
                                    0x00,
                                    0x00,
                                    0xBF,
                                    0x00,
                                    0x00,
                                    0x00,
                                    0x00,
                                    0x00,
                                    0x00,
                                    0x80,
                                    0x3F,
                                    0x00,
                                    0x00,
                                    0x00,
                                    0x00,
                                    0x00,
                                    0x00,
                                    0x80,
                                    0x3F,
                                    0x00,
                                    0x00,
                                    0x00,
                                    0xBF,
                                    0x00,
                                    0x00,
                                    0x00,
                                    0xBF,
                                    0x00,
                                    0x00,
                                    0x00,
                                    0x00,
                                    0x00,
                                    0x00,
                                    0x00,
                                    0x00,
                                    0x00,
                                    0x00,
                                    0x80,
                                    0x3F,
                                    0x00,
                                    0x00,
                                    0x80,
                                    0x3F
                                ],
                                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                            }),
                            index_buffer: device.create_buffer(&BufferDescriptor {
                                label: None,
                                size: 0,
                                usage: BufferUsages::COPY_DST,
                                mapped_at_creation: false,
                            }),
                        }),
                        material: ModelMaterial::Normal(NormalMaterial {}),
                        transform: Affine3A::IDENTITY,
                    }))],
                })),
            }))],
            instance,
            adapter,
            device,
            queue,
            surface,
            sconfig,
            window,
        }
    }

    pub fn render_all(&mut self) {
        let context = RenderContext {
            device: &self.device,
            queue: &self.queue,
            instance: &self.instance,
            adapter: &self.adapter,
        };

        let mut encoder = self
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("Frame command encoder"),
            });

        let surface_texture = self.surface.get_current_texture().unwrap();
        let window_view = surface_texture.texture.create_view(&TextureViewDescriptor {
            label: Some("Surface window"),
            format: Some(self.sconfig.format),
            dimension: Some(TextureViewDimension::D2),
            aspect: TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        });

        for viewport_rc in &self.viewports {
            let viewport = viewport_rc.borrow();
            let scenario = viewport.scenario.borrow();

            let target_size = viewport.size.as_vec2();

            viewport
                .camera
                .update_buffer(target_size.x / target_size.y, &self.queue);

            if let Some(clear_color) = viewport.clear_color {
                let pass = encoder.begin_render_pass(&RenderPassDescriptor {
                    label: Some("Color clear pass"),
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: todo!(),
                        resolve_target: None,
                        ops: Operations {
                            load: LoadOp::Clear(Color {
                                r: clear_color.x as f64,
                                g: clear_color.y as f64,
                                b: clear_color.z as f64,
                                a: clear_color.w as f64,
                            }),
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: None,
                });
                drop(pass);
            }

            for instance_rc in &scenario.instances {
                let instance = instance_rc.borrow();
                match &instance.material {
                    ModelMaterial::Normal(material) => {
                        material.render(&instance, context, &viewport, &mut encoder)
                    }
                }
            }
        }

        self.queue.submit(iter::once(encoder.finish()));
        surface_texture.present();
    }
}

#[derive(Clone, Copy)]
pub struct RenderContext<'a> {
    pub device: &'a Device,
    pub queue: &'a Queue,
    pub instance: &'a Instance,
    pub adapter: &'a Adapter,
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

pub struct Viewport {
    pub size: UVec2,
    pub color_target: Rc<Texture>,
    pub clear_color: Option<Vec4>,
    pub camera: Camera,
    pub scenario: Rc<RefCell<Scenario>>,
}

pub enum Texture {
    Window(),
    Texture2D(Texture2D),
}

pub struct Texture2D {
    pub view: TextureView,
    pub format: TextureFormat,
    pub size: UVec2,
    pub mipmaps: u32,
}


pub struct Scenario {
    instances: Vec<Rc<RefCell<ModelInstance>>>,
}

impl Scenario {
    pub fn new() -> Self {
        Self { instances: vec![] }
    }
}

pub struct Model {
    // TODO: make this struct more matching for BF2 models
    name: String,

    // Meaning of these fields is defined by the ModelMaterial
    vertex_buffer: Buffer,
    index_buffer: Buffer,
}

pub struct NormalMaterial {}

impl NormalMaterial {
    pub fn render(
        &self,
        instance: &ModelInstance,
        context: RenderContext,
        viewport: &Viewport,
        encoder: &mut CommandEncoder,
    ) {
        let RenderContext { device, .. } = context;

        static PIPELINE_CELL: OnceCell<RenderPipeline> = OnceCell::new();
        let pipeline = PIPELINE_CELL.get_or_init(|| {
            let shader = include_shader!(device, "example_triangle.shader");
            device.create_render_pipeline(&RenderPipelineDescriptor {
                label: None,
                layout: Some(&device.create_pipeline_layout(&PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[],
                    push_constant_ranges: &[],
                })),
                vertex: VertexState {
                    module: &shader.module,
                    entry_point: "main",
                    buffers: &[VertexBufferLayout {
                        array_stride: (2 + 4) * 4,
                        step_mode: VertexStepMode::Vertex,
                        attributes: &[
                            VertexAttribute {
                                format: VertexFormat::Float32x2,
                                offset: 0 * 4,
                                shader_location: 0,
                            },
                            VertexAttribute {
                                format: VertexFormat::Float32x4,
                                offset: 2 * 4,
                                shader_location: 1,
                            },
                        ],
                    }],
                },
                fragment: Some(FragmentState {
                    module: &shader.module,
                    entry_point: "main",
                    targets: &[todo!()],
                }),
                primitive: PrimitiveState {
                    topology: PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: FrontFace::Cw,
                    cull_mode: Some(Face::Back),
                    polygon_mode: PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: MultisampleState::default(),
                multiview: None,
            })
        });

        let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
            label: None,
            color_attachments: todo!(),
            depth_stencil_attachment: None,
        });
        pass.set_vertex_buffer(0, instance.model.vertex_buffer.slice(..));
        pass.set_pipeline(pipeline);
        pass.draw(0..3, 0..1);
    }
}

pub enum ModelMaterial {
    Normal(NormalMaterial),
}

pub struct ModelInstance {
    model: Rc<Model>,
    material: ModelMaterial,
    transform: Affine3A,
}
