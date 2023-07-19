use crate::{
    devui::{imgui_ext::UiExt, viewers::model_preview::ModelPreview},
    graphics::{CubemapHandle, TextureHandle},
    scene::EngineBorrow,
};
use glam::*;
use imgui::Ui;

pub(super) struct TextureViewer {
    selected_texture: Option<(SelectedTexture, ModelPreview)>,
}

impl Default for TextureViewer {
    fn default() -> Self {
        Self {
            selected_texture: None,
        }
    }
}

impl super::RendererViewerTab for TextureViewer {
    fn show_ui(&mut self, ui: &Ui, engine: &mut EngineBorrow) {
        let renderer = &mut engine.renderer;

        let mut texture_to_select = None;

        ui.child_window("left")
            .border(true)
            .horizontal_scrollbar(true)
            .size([200.0, 0.0])
            .build(|| {
                for (texture, handle) in renderer.textures.iter_handles() {
                    if ui
                        .selectable_config(&texture.label)
                        .selected(self.selected_texture.matches_texture(&handle))
                        .build()
                    {
                        texture_to_select = Some(SelectedTexture::Texture(handle));
                    }
                }
                ui.separator();
                for (texture, handle) in renderer.cubemaps.iter_handles() {
                    if ui
                        .selectable_config(&texture.label)
                        .selected(self.selected_texture.matches_cubemap(&handle))
                        .build()
                    {
                        texture_to_select = Some(SelectedTexture::Cubemap(handle));
                    }
                }
            });

        ui.same_line();

        let _group_token = ui.begin_group();

        let preview_size = [0.0, -ui.text_line_height() * 8.0];

        match &mut self.selected_texture {
            Some((SelectedTexture::Texture(handle), model_preview)) => {
                ui.text(&renderer.textures.get(handle).label);
                ui.same_line();
                ui.text_disabled("(2D texture)");
                ui.separator();
                
                model_preview.display(ui, renderer, preview_size);
                
                let texture = renderer.textures.get(handle);
                if let Some(_) = ui.begin_table("TextureInfo", 2) {
                    ui.table_next_row();
                    ui.table_set_column_index(0);
                    ui.text_disabled("Size");
                    ui.table_next_column();
                    ui.text(format!(
                        "{}x{}",
                        texture.handle.width(),
                        texture.handle.height(),
                    ));

                    ui.table_next_row();
                    ui.table_set_column_index(0);
                    ui.text_disabled("Live references");
                    ui.table_next_column();
                    ui.text(format!("{} (+1)", handle.live_references() - 1));
                    ui.same_line();
                    ui.question_tooltip("The `+1` is the reference used by the texture viewer");

                    ui.table_next_row();
                    ui.table_set_column_index(0);
                    ui.text_disabled("Mipmaps");
                    ui.table_next_column();
                    ui.text(format!("{}", texture.handle.mip_level_count()));

                    ui.table_next_row();
                    ui.table_set_column_index(0);
                    ui.text_disabled("Format");
                    ui.table_next_column();
                    ui.text(format!("{:?}", texture.handle.format()));
                    if let Some(d3dformat) = texture.d3d_format {
                        ui.same_line();
                        ui.text_disabled(format!("(D3DFMT_{:?})", d3dformat));
                        if ui.is_item_hovered() {
                            ui.tooltip_text(
                                "The original Direct3D texture format as loaded by the engine.",
                            );
                        }
                    }

                    ui.table_next_row();
                    ui.table_set_column_index(0);
                    ui.text_disabled("Filtered");
                    ui.same_line();
                    ui.question_tooltip("Specifies whether linear texture sampling is enabled or disabled for this texture.");
                    ui.table_next_column();
                    ui.text(match texture.unfiltered {
                        true => "no",
                        false => "yes",
                    });
                }
            }
            Some((SelectedTexture::Cubemap(handle), model_preview)) => {
                let cubemap = renderer.cubemaps.get(&handle);

                ui.text(&cubemap.label);
                ui.same_line();
                ui.text_disabled("(cubemap texture)");
                ui.separator();

                model_preview.display(ui, renderer, preview_size);
            }
            None => ui.text_disabled("(No texture selected)"),
        }

        if let Some(texture_to_select) = texture_to_select {
            self.selected_texture = Some((texture_to_select, ModelPreview::new(renderer)));
        }
    }
}

#[derive(PartialEq)]
enum SelectedTexture {
    Texture(TextureHandle),
    Cubemap(CubemapHandle),
}

trait SelectedTextureExt {
    fn matches_texture(&self, other: &TextureHandle) -> bool;
    fn matches_cubemap(&self, other: &CubemapHandle) -> bool;
}

impl SelectedTextureExt for Option<(SelectedTexture, ModelPreview)> {
    fn matches_texture(&self, other: &TextureHandle) -> bool {
        match self {
            Some((SelectedTexture::Texture(handle), _)) => handle == other,
            _ => false,
        }
    }

    fn matches_cubemap(&self, other: &CubemapHandle) -> bool {
        match self {
            Some((SelectedTexture::Cubemap(handle), _)) => handle == other,
            _ => false,
        }
    }
}
