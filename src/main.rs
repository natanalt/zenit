//! Zenit Engine entry point

pub mod engine;
pub mod lua;
pub mod munge;
pub mod resources;
pub mod renderer;
pub mod utils;
pub mod console;
pub mod shell;
pub mod args;

pub type AnyResult<T> = anyhow::Result<T>;

fn main(){
    simple_logger::SimpleLogger::new()
        .with_module_level("wgpu_hal", log::LevelFilter::Off)
        .with_module_level("wgpu_core", log::LevelFilter::Off)
        .with_colors(true)
        .init()
        .expect("couldn't init logger");
    engine::run();
}
