use gtk4::Orientation;
use super::View;

pub struct FilesView {
    root: gtk4::Box,
}

impl Default for FilesView {
    fn default() -> Self {
        let root = gtk4::Box::builder()
            .orientation(Orientation::Horizontal)
            .build();
        
        Self {
            root,
        }
    }
}

impl View for FilesView {
    fn get_id(&self) -> &str {
        "files_view"
    }

    fn get_title(&self) -> &str {
        "Files"
    }

    fn get_root(&self) -> &gtk4::Box {
        &self.root
    }
}
