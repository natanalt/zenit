use imgui::Ui;

use crate::{entities::Universe, scene::EngineBorrow};
use super::{DevUiComponent, DevUiTextures};

pub fn add_menu_bar(uni: &mut Universe) {
    uni.create_entity_with(DevUiComponent {
        widget: Some(Box::new(menu_bar)),
    });
}

fn menu_bar(ui: &mut Ui, engine: &mut EngineBorrow, _: &mut DevUiTextures) -> bool {
    let universe = &mut engine.universe;

    ui.main_menu_bar(|| {
        ui.text_disabled("Zenit Engine");
        if ui.is_item_hovered() {
            ui.tooltip(|| {
                if let Some(_) = ui.begin_table("ZenitBuildInfo", 2) {
                    ui.table_next_row();
                    ui.table_set_column_index(0);
                    ui.text_disabled("Version");
                    ui.table_next_column();
                    ui.text(crate::VERSION);

                    ui.table_next_row();
                    ui.table_set_column_index(0);
                    ui.text_disabled("Commit");
                    ui.table_next_column();
                    ui.text(concat!(env!("VERGEN_GIT_SHA"), " (", env!("VERGEN_GIT_BRANCH"), " branch)"));

                    ui.table_next_row();
                    ui.table_set_column_index(0);
                    ui.text_disabled("Compiler");
                    ui.table_next_column();
                    ui.text(concat!("rustc ", env!("VERGEN_RUSTC_SEMVER"), " (", env!("VERGEN_RUSTC_CHANNEL"), ")"));

                    ui.table_next_row();
                    ui.table_set_column_index(0);
                    ui.text_disabled("Build date");
                    ui.table_next_column();
                    ui.text(env!("VERGEN_BUILD_TIMESTAMP"));
                }
            });
        }

        ui.separator();

        ui.menu("Tools", || {
            for (name, callback) in super::viewers::TOOLS {
                if ui.menu_item(name) {
                    (callback)(universe);
                }
            }
        });
    });

    true
}
