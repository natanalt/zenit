//! Zenit Engine entry point

pub mod engine;
pub mod level_data;
pub mod lua;
pub mod namespace;
pub mod renderer;
pub mod utils;
pub mod console;

pub type AnyResult<T> = anyhow::Result<T>;

fn main(){
    simple_logger::SimpleLogger::new()
        .with_colors(true)
        .init()
        .expect("couldn't init logger");
    engine::run();
}
