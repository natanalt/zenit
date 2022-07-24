use gtk4::{prelude::*, Application, ApplicationWindow, Button, Orientation, Stack, StackSidebar};

pub mod views;

fn main() {
    gtk4::init().unwrap();

    let app = Application::builder()
        .application_id("zenit_engine.viewer")
        .build();

    // 😳😳😳
    let views = views::get_default_views().leak();

    app.connect_activate(|app| {
        let stack = Stack::new();
        for view in views.iter() {
            stack.add_titled(view.get_root(), Some(view.get_id()), view.get_title());
        }

        let root = gtk4::Box::builder()
            .orientation(Orientation::Horizontal)
            .build();

        root.append(&StackSidebar::builder().stack(&stack).build());
        root.append(&stack);

        ApplicationWindow::builder()
            .application(app)
            .default_width(1000)
            .default_height(700)
            .title("Zenit Data Viewer")
            .child(&root)
            .build()
            .present();
    });

    app.run();
}
