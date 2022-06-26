use super::{ext::{BevyCommandsExtensions, EguiUiExtensions}, Widget};
use crate::{root::GameRoot, schedule::TopFrameStage};
use bevy_ecs::prelude::*;
use std::path::PathBuf;

pub fn init(_world: &mut World, schedule: &mut Schedule) {
    schedule.add_system_to_stage(TopFrameStage::ControlPanel, root_select);
}

#[derive(Bundle, Default)]
pub struct RootSelectWindowBundle {
    pub widget: Widget,
    pub rsw: RootSelectWindow,
}

#[derive(Component, Default)]
pub struct RootSelectWindow {
    pub path_buffer: String,
}

pub fn root_select(
    mut commands: Commands,
    mut query: Query<(Entity, &mut RootSelectWindow)>,
    ctx: ResMut<egui::Context>,
) {
    if query.is_empty() {
        return;
    }

    let (entity, mut window) = query
        .get_single_mut()
        .expect("there can be only one game root window");

    egui::Window::new("Select Game Root")
        .resizable(false)
        .title_bar(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(&ctx, |ui| {
            ui.label("Please select Star Wars Battlefront II root directory.");
            ui.e_faint_label("(The one that contains a GameData directory)");
            ui.add_space(5.0);
            ui.text_edit_singleline(&mut window.path_buffer);
            ui.add_space(5.0);
            ui.e_faint_label(
                "It can also be specified with the --game-root command line parameter.",
            );
            ui.add_space(5.0);
            if ui.button("OK").clicked() {
                let new_root = GameRoot::new(Some(PathBuf::from(&window.path_buffer)).as_ref());
                if new_root.is_invalid() {
                    commands.e_error_box("Error", "Invalid root directory");
                } else {
                    commands.insert_resource(new_root);
                    commands.entity(entity).despawn();
                    //response.add_widget(SideView);
                    //response.add_widget(TopView);
                }
            }
        });
}
