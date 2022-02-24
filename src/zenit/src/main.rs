use bevy::prelude::*;

fn main() {
    // This is temporary.
    // I'll massively cut down on Bevy's defaults, as I don't need the entire engine
    App::new()
        .insert_resource(WindowDescriptor {
            width: 1280.0,
            height: 720.0,
            title: "Zenit Engine".to_string(),
            vsync: true,
            resizable: true,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .run();
}
