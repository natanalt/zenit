use std::{sync::Arc, ops::Deref};
use winit::window::Window;

/// Handle to the main window
pub struct MainWindow(pub Arc<Window>);

impl Deref for MainWindow {
    type Target = Window;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

// Window events may be placed here someday
