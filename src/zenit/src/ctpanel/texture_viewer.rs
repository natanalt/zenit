use super::ext::EguiUiExtensions;
use crate::{
    render::{
            texture::{Texture2D, TextureSource},
            RenderContext,
        RenderCommands,
    },
    schedule::TopFrameStage,
};
use bevy_ecs::prelude::*;
use byteorder::{NativeEndian, WriteBytesExt};
use wgpu::util::DeviceExt;

pub fn init(_world: &mut World, schedule: &mut Schedule) {
    schedule.add_system_to_stage(TopFrameStage::ControlPanel, texture_viewer_render);
    schedule.add_system_to_stage(
        TopFrameStage::ControlPanel,
        texture_viewer.after(texture_viewer_render),
    );
}

#[derive(Bundle, Default)]
pub struct TextureViewerBundle {
    pub window: TextureViewerWindow,
}

struct RenderState {
    pipeline: wgpu::RenderPipeline,
    target: Texture2D,
    vertices: wgpu::Buffer,
}

#[derive(Component, Default)]
pub struct TextureViewerWindow {
    texture_id: Option<egui::TextureId>,
    render_state: Option<RenderState>,
}

pub fn texture_viewer_render(
    mut query: Query<&mut TextureViewerWindow>,
    mut rcom: ResMut<RenderCommands>,
    mut rpass: NonSendMut<super::EguiRenderPass>,
    ctx: Res<RenderContext>,
) {
    for mut window in query.iter_mut() {
        if window.render_state.is_none() {
            window.render_state = Some({
                let shader = crate::include_shader!(ctx.device, "example_triangle.shader");
                let target = TextureSource::new(
                    &ctx,
                    &wgpu::TextureDescriptor {
                        label: None,
                        size: wgpu::Extent3d {
                            width: 640,
                            height: 480,
                            depth_or_array_layers: 1,
                        },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: wgpu::TextureDimension::D2,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                            | wgpu::TextureUsages::TEXTURE_BINDING,
                    },
                )
                .into_2d();

                window.texture_id = Some(rpass.register_native_texture(
                    &ctx.device,
                    &target.view,
                    wgpu::FilterMode::Nearest,
                ));

                RenderState {
                    pipeline: ctx
                        .device
                        .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                            label: Some("test"),
                            layout: None,
                            vertex: wgpu::VertexState {
                                module: &shader.module,
                                entry_point: "main",
                                buffers: &[crate::single_vertex_buffer![
                                    0 => Float32x2,
                                    1 => Float32x4,
                                ]],
                            },
                            fragment: Some(wgpu::FragmentState {
                                module: &shader.module,
                                entry_point: "main",
                                targets: &[wgpu::TextureFormat::Rgba8Unorm.into()],
                            }),
                            primitive: crate::render::utils::USUAL_PRIMITIVES,
                            depth_stencil: None,
                            multisample: wgpu::MultisampleState::default(),
                            multiview: None,
                        }),
                    target,
                    vertices: ctx
                        .device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: None,
                            usage: wgpu::BufferUsages::VERTEX,
                            contents: {
                                #[rustfmt::skip]
                                const DATA: &[f32] = &[
                                     0.0,  0.5, 1.0, 0.0, 0.0, 1.0,
                                     0.5, -0.5, 0.0, 1.0, 0.0, 1.0,
                                    -0.5, -0.5, 0.0, 0.0, 1.0, 1.0,
                                ];

                                let mut data = Vec::with_capacity(DATA.len() * 4);
                                let mut cursor = std::io::Cursor::new(&mut data);

                                for &value in DATA {
                                    cursor.write_f32::<NativeEndian>(value).unwrap();
                                }

                                // may it leak uwu
                                data.leak()
                            },
                        }),
                }
            });
        }

        let render_state = window.render_state.as_ref().unwrap();

        let mut encoder = ctx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: &render_state.target.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });
        pass.set_pipeline(&render_state.pipeline);
        pass.set_vertex_buffer(0, render_state.vertices.slice(..));
        pass.draw(0..3, 0..1);
        drop(pass);

        rcom.push(encoder.finish());
    }
}

pub fn texture_viewer(
    mut commands: Commands,
    mut query: Query<(Entity, &mut TextureViewerWindow)>,
    ctx: ResMut<egui::Context>,
) {
    for (entity, mut window) in query.iter() {
        let mut visible = true;
        egui::Window::new("Texture Viewer")
            .open(&mut visible)
            .default_size([500.0, 400.0])
            .show(&ctx, |ui| {
                egui::SidePanel::left("texture_viewer_side").show_inside(ui, |ui| {
                    ui.label("nya");
                });

                ui.horizontal(|ui| {
                    if ui.button("Close file").clicked() {
                        // ...
                    }

                    ui.e_faint_label("side/all.lvl");
                });
                ui.separator();

                egui::Frame::dark_canvas(ui.style())
                    .inner_margin(1.0)
                    .show(ui, |ui| {
                        ui.image(
                            window.texture_id.unwrap(),
                            ui.available_size() / egui::vec2(1.0, 1.5),
                        );
                    });
            });

        if !visible {
            commands.entity(entity).despawn();
        }
    }
}
