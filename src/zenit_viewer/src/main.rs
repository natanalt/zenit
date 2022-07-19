use gtk4::{prelude::*, Application, ApplicationWindow, Button};

fn main() {
    let app = Application::builder()
        .application_id("zenit_engine.viewer")
        .build();
    
    app.connect_activate(|app| {
        let button = Button::builder()
            .label("nya")
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();
        
        button.connect_clicked(move |_| {
            println!("nya");
        });

        ApplicationWindow::builder()
            .application(app)
            .title("Zenit Data Viewer")
            .child(&button)
            .build()
            .present();
    });

    app.run();
}
