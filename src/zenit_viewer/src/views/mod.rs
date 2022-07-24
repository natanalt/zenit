
mod files_view;
mod textures_view;

pub trait View {
    fn get_id(&self) -> &str;
    fn get_title(&self) -> &str;
    fn get_root(&self) -> &gtk4::Box;
}

pub fn get_default_views() -> Vec<Box<dyn View>> {
    vec![
        Box::new(files_view::FilesView::default()),
        Box::new(textures_view::TexturesView::default()),
    ]
}
