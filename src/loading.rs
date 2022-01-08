//! Implementation of the loading screen

use log::info;
use std::path::PathBuf;
use wgpu::TextureViewDescriptor;

use crate::{
    engine::{Engine, GameState},
    resources::{Resources, ContainerKind},
    shell::ShellState,
};

#[derive(Debug, Clone, Copy)]
pub enum LoadingTarget {
    Shell,
    Ingame,
}

#[derive(Debug, Clone)]
pub struct LoadingState {
    pub target: LoadingTarget,
    pub files_to_load: Vec<(ContainerKind, PathBuf)>,
}

impl LoadingState {
    pub fn shell_loader(resources: &Resources) -> Self {
        Self {
            target: LoadingTarget::Shell,
            files_to_load: vec![
                (ContainerKind::Persistent, resources.data_path("core.lvl")),
                (ContainerKind::Persistent, resources.data_path("common.lvl")),
                (ContainerKind::Persistent, resources.data_path("shell.lvl")),
            ],
        }
    }

    pub fn frame(mut self, engine: &mut Engine) -> GameState {
        // TODO: multithreaded/asynchronous loading

        info!("Loading new state...");

        for (target, path) in self.files_to_load.drain(0..) {
            engine
                .resources
                .load_file(path, target)
                .expect("Failed to load file");
        }

        // Make sure to always send *something* to queue
        let renderer = engine.renderer.as_mut().unwrap();
        let mut encoder = renderer
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &renderer
                    .surface_texture
                    .as_ref()
                    .unwrap()
                    .texture
                    .create_view(&TextureViewDescriptor::default()),
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        renderer.buffers.push(encoder.finish());

        match self.target {
            LoadingTarget::Shell => GameState::Shell(ShellState::new(engine)),
            LoadingTarget::Ingame => todo!(),
        }
    }
}
