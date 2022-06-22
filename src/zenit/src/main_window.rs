use std::sync::Arc;
use winit::window::Window;
use zenit_proc::TupledContainerDerefs;

/// Handle to the main window
#[derive(TupledContainerDerefs)]
pub struct MainWindow(pub Arc<Window>);

// Window events may be placed here someday
