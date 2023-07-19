use crate::graphics::{
    CameraDescriptor, CameraHandle, Renderer, SceneBuilder, SkyboxDescriptor, SkyboxHandle,
};
use glam::{uvec2, vec4, Affine3A};
use imgui::{TextureId, Ui};
use std::sync::Arc;
use zenit_utils::math::AngleExt;

pub struct ModelPreview {
    texture_id: Arc<TextureId>,
    camera: CameraHandle,
    skybox: SkyboxHandle,
    cached_size: [f32; 2],
    id: u64,
}

impl ModelPreview {
    pub fn new(renderer: &mut Renderer) -> Self {
        let camera = renderer.create_camera(&CameraDescriptor {
            texture_size: uvec2(1, 1),
            fov: 90.degrees(),
            near_plane: 0.0001,
            far_plane: 1000.0,
        });

        let texture_id = renderer.add_imgui_camera(camera.clone());

        let skybox = renderer.create_skybox(&SkyboxDescriptor {
            name: String::from("model preview skybox"),
            background: crate::graphics::SkyboxBackground::Solid(vec4(1.0, 0.0, 1.0, 1.0)),
        });

        Self {
            texture_id,
            camera,
            skybox,
            cached_size: [-1.0, -1.0],
            id: zenit_utils::counter::next(),
        }
    }

    pub fn display(&mut self, ui: &Ui, renderer: &mut Renderer, [width, height]: [f32; 2]) {
        let mut should_render_scene = true;

        // Display the texture in Dear ImGui
        ui.child_window(format!("ModelPreview##{}", self.id))
            .border(true)
            .scroll_bar(false)
            .horizontal_scrollbar(false)
            .size([width, height])
            .build(|| {
                let content_size = ui.content_region_avail();
                let content_scale = ui.io().display_framebuffer_scale;
                let framebuffer_size = uvec2(
                    (content_size[0] * content_scale[0]) as u32,
                    (content_size[1] * content_scale[1]) as u32,
                );

                if framebuffer_size.x == 0 || framebuffer_size.y == 0 {
                    should_render_scene = false;
                    return;
                }

                if content_size != self.cached_size {
                    self.cached_size = content_size;
                    let camera = renderer.cameras.get_mut(&self.camera);
                    camera.resize(&renderer.dc, framebuffer_size);
                }

                imgui::Image::new(*self.texture_id, content_size).build(ui);
            });

        // Enqueue the render
        if should_render_scene {
            let mut scene = SceneBuilder::new(renderer);
            scene.set_skybox(&self.skybox);
            scene.render_to(&self.camera, Affine3A::IDENTITY);
            scene.submit();
        }
    }
}
