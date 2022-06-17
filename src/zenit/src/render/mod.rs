use self::base::{texture::Texture2D, RenderContext};
use anyhow::anyhow;
use base::surface::MainWindow;
use glam::UVec2;
use pollster::FutureExt;
use std::sync::Arc;
use winit::window::Window;
use zenit_utils::{ok, AnyResult};

pub mod base;
pub mod pipelines;

pub struct Renderer {
    pub context: Arc<RenderContext>,
    pub main_window: MainWindow,
    pub builtin_textures: BuiltinTextures,
}

impl Renderer {
    pub fn new(window: Arc<Window>) -> AnyResult<Self> {
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(&window) };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .block_on()
            .ok_or(anyhow!("Couldn't find a graphics device"))?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::downlevel_defaults(),
                },
                None,
            )
            .block_on()?;

        let context = Arc::new(RenderContext {
            device,
            queue,
            adapter_info: adapter.get_info(),
            adapter,
        });

        Ok(Self {
            main_window: MainWindow::new(&context, surface, window),
            builtin_textures: BuiltinTextures::new(&context),
            context,
        })
    }

    pub fn begin_frame(&mut self) {
        self.main_window.begin_frame(&self.context);
    }

    pub fn finish_frame(&mut self, buffers: Vec<wgpu::CommandBuffer>) -> AnyResult {
        assert!(
            !buffers.is_empty(),
            "You need to submit at least one command buffer"
        );

        self.context.queue.submit(buffers);
        self.main_window.finish_frame();

        ok()
    }
}

pub struct BuiltinTextures {
    pub not_found: Arc<Texture2D>,
    pub corrupted: Arc<Texture2D>,
}

impl BuiltinTextures {
    pub fn new(context: &RenderContext) -> Self {
        Self {
            not_found: Arc::new(Texture2D::from_rgba8(
                context,
                UVec2::new(256, 256),
                &make_checkered_texture(256, 64, [0, 0, 0], [255, 0, 255]),
            )),
            corrupted: Arc::new(Texture2D::from_rgba8(
                context,
                UVec2::new(256, 256),
                &make_checkered_texture(256, 64, [0, 0, 0], [255, 0, 0]),
            )),
        }
    }
}

fn make_checkered_texture(size: usize, square_size: usize, ca: [u8; 3], cb: [u8; 3]) -> Vec<u8> {
    let mut result = Vec::with_capacity(size * size * 4);

    for y in 0..size {
        let flip = y % 2 == 1;
        for x in 0..size {
            let mut side = (x / square_size) % 2;
            if flip {
                side = if side == 0 { 1 } else { 0 };
            }

            if side == 0 {
                result.push(ca[0]);
                result.push(ca[1]);
                result.push(ca[2]);
                result.push(255);
            } else {
                result.push(cb[0]);
                result.push(cb[1]);
                result.push(cb[2]);
                result.push(255);
            }
        }
    }

    result
}
