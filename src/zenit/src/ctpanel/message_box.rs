use bevy_ecs::prelude::*;
use crate::schedule::TopFrameStage;

pub fn init(_world: &mut World, schedule: &mut Schedule) {
    schedule.add_system_to_stage(TopFrameStage::ControlPanel, message_box);
}

#[derive(Component)]
pub struct MessageBox {
    pub id: egui::Id,
    pub title: String,
    pub message: String,
}

impl MessageBox {
    // Currently those functions are identical, they will be changed someday
    // TODO: icons in message boxes (and make them prettier in general)

    pub fn info(title: &str, message: &str) -> Self {
        Self {
            id: egui::Id::new(zenit_utils::counter::next()),
            title: title.to_string(),
            message: message.to_string(),
        }
    }

    pub fn warn(title: &str, message: &str) -> Self {
        Self::info(title, message)
    }

    pub fn error(title: &str, message: &str) -> Self {
        Self::info(title, message)
    }
}

pub fn message_box(
    mut commands: Commands,
    mut boxes: Query<(Entity, &MessageBox)>,
    ctx: ResMut<egui::Context>,
) {
    for (entity, mbox) in boxes.iter_mut() {
        let mut opened = true;
        let mut ok_pressed = false;
        egui::Window::new(&mbox.title)
            .id(mbox.id)
            .resizable(false)
            .open(&mut opened)
            .show(&ctx, |ui| {
                ui.label(&mbox.message);
                ui.separator();
                ok_pressed = ui.button("OK").clicked();
            });

        if ok_pressed || !opened {
            commands.entity(entity).despawn();
        }
    }
}
