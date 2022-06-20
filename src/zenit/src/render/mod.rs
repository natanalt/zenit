use crate::{main_window::MainWindow, schedule::TopFrameStage};
use bevy_ecs::prelude::*;

use self::base::{texture::Texture2D, RenderContext};
use anyhow::anyhow;
use base::surface::RenderWindow;
use glam::UVec2;
use pollster::FutureExt;
use std::{sync::Arc, ops::{DerefMut, Deref}, mem};
use winit::window::Window;
use zenit_utils::{ok, AnyResult};

pub mod base;
pub mod pipelines;

pub struct RenderBuffers(pub Vec<wgpu::CommandBuffer>);

impl Deref for RenderBuffers {
    type Target = Vec<wgpu::CommandBuffer>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RenderBuffers {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub fn init(world: &mut World, schedule: &mut Schedule) {
    let window = world
        .get_resource::<MainWindow>()
        .expect("main window not found")
        .0
        .clone();
    
    world.insert_resource(Renderer::new(window).expect("couldn't init the renderer"));
    schedule.add_system_to_stage(TopFrameStage::RenderStart, begin_frame_system);
    schedule.add_system_to_stage(TopFrameStage::RenderFinish, finish_frame_system);
}

pub fn begin_frame_system(mut renderer: ResMut<Renderer>) {
    renderer.begin_frame();
}

pub fn finish_frame_system(
    mut buffers: ResMut<RenderBuffers>,
    mut renderer: ResMut<Renderer>,
) {
    renderer.finish_frame(mem::take(&mut buffers)).unwrap();
}

pub struct Renderer {
    pub context: Arc<RenderContext>,
    pub main_window: RenderWindow,
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
            main_window: RenderWindow::new(&context, surface, window),
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
