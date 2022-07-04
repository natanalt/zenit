//! Core of Zenit's very good:tm: renderer
//! 
//! ## Renderer, ECS, and you
//! The renderer exposes several resources for use by systems:
//!  * [`RenderCommands`] - a vector of wgpu command buffers, to be submitted
//!    to the GPU at the frame's end
//!  * [`BuiltinTextures`] - what the name suggests
//!  * [`RenderWindow`] - main window's wgpu stuff
//!  * [`PipelineStorage`] - stores pipelines
//!  * [`RenderContext`] - gives access to [`wgpu::Device`], [`wgpu::Queue`] and more.
//!    Doesn't have to be locked - can be use immutably.

use crate::{main_window::MainWindow, schedule::TopFrameStage, render::pipelines::PipelineStorage};
use surface::RenderWindow;
use bevy_ecs::prelude::*;
use glam::*;
use log::*;
use pollster::FutureExt;
use zenit_proc::TupledContainerDerefs;
use std::{mem, sync::Arc};

use self::texture::Texture2D;

pub mod pipelines;
pub mod scene;
pub mod shaders;
pub mod surface;
pub mod texture;
pub mod utils;
pub mod mipmaps;

/// Shared structure containing various wgpu drawing objects
pub struct RenderContext {
    pub device: wgpu::Device,
    pub adapter: wgpu::Adapter,
    pub adapter_info: wgpu::AdapterInfo,
    pub queue: wgpu::Queue,
}

#[derive(Default, TupledContainerDerefs)]
pub struct RenderCommands(pub Vec<wgpu::CommandBuffer>);

pub fn init(world: &mut World, schedule: &mut Schedule) {
    schedule.add_system_to_stage(TopFrameStage::FrameStart, begin_frame_system);
    schedule.add_system_to_stage(TopFrameStage::FrameFinish, finish_frame_system);

    let window = world
        .get_resource::<MainWindow>()
        .expect("main window not found")
        .0
        .clone();

    let instance = wgpu::Instance::new(wgpu::Backends::all());
    let surface = unsafe { instance.create_surface(&window) };

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        })
        .block_on()
        .expect("couldn't find a graphics device");

    let adapter_info = adapter.get_info();
    info!("Using {}", &adapter_info.name);

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::downlevel_defaults(),
            },
            None,
        )
        .block_on()
        .expect("couldn't init the graphics device");

    let context = RenderContext {
        device,
        queue,
        adapter_info,
        adapter,
    };

    world.init_resource::<RenderCommands>();
    world.init_resource::<PipelineStorage>();
    world.insert_resource(BuiltinTextures::new(&context));
    world.insert_resource(RenderWindow::new(&context, surface, window));
    world.insert_resource(context);
}

pub fn begin_frame_system(context: Res<RenderContext>, mut main_window: ResMut<RenderWindow>) {
    main_window.begin_frame(&context);
}

pub fn finish_frame_system(
    context: Res<RenderContext>,
    mut main_window: ResMut<RenderWindow>,
    mut buffers: ResMut<RenderCommands>,
) {
    assert!(!buffers.is_empty(), "cannot submit no graphics commands");
    context.queue.submit(mem::take(&mut buffers.0));
    main_window.finish_frame();
}

pub struct BuiltinTextures {
    /// Used as a default for non-existent textures.
    /// Visually it looks like the Source engine magenta-black checker pattern
    pub not_found: Arc<Texture2D>,
    /// Used for textures that exist, but cannot be loaded for some reason.
    /// Visually it's a red-black checker pattern.
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
