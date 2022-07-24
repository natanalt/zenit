use gtk4::{prelude::*, Orientation, Label, PolicyType, ScrolledWindow, ListBox, GLArea};
use super::View;

pub struct TexturesView {
    root: gtk4::Box,
    texture_list: ListBox,
}

impl Default for TexturesView {
    fn default() -> Self {
        let texture_list = ListBox::builder()
            .width_request(150)
            .build();
        texture_list.append(&Label::new(Some("white")));
        texture_list.append(&Label::new(Some("att_main")));
        texture_list.append(&Label::new(Some("whatever_else")));
        texture_list.append(&Label::new(Some("owo")));
        
        let texture_list_scroll = ScrolledWindow::builder()
            .hscrollbar_policy(PolicyType::Never)
            .min_content_width(150)
            .child(&texture_list)
            .build();
        
        let gl_view = GLArea::builder()
            .height_request(500)
            .build();
        
        let format_list = ListBox::new();
        format_list.append(&Label::new(Some("DXT1")));
        format_list.append(&Label::new(Some("DXT3")));
        format_list.append(&Label::new(Some("R8G8B8A8")));

        let info_container = gtk4::Frame::builder().build();

        let details_container = gtk4::Box::builder()
            .orientation(Orientation::Horizontal)
            .build();
        details_container.append(&format_list);
        details_container.append(&info_container);
        
        let main_container = gtk4::Box::builder()
            .margin_start(10)
            .margin_end(10)
            .margin_top(10)
            .margin_bottom(10)
            .orientation(Orientation::Vertical)
            .build();
        main_container.append(&gl_view);
        main_container.append(&details_container);

        let root = gtk4::Box::builder()
            .orientation(Orientation::Horizontal)
            .build();
        root.append(&texture_list_scroll);
        root.append(&main_container);

        Self {
            root,
            texture_list,
        }
    }
}

impl View for TexturesView {
    fn get_id(&self) -> &str {
        "textures_view"
    }

    fn get_title(&self) -> &str {
        "Textures"
    }

    fn get_root(&self) -> &gtk4::Box {
        &self.root
    }
}
