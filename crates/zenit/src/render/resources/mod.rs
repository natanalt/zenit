//! A render resource is a broad term for something that holds GPU resources.
//! 
//! For example, a camera holds its own uniform buffer. A model holds its own buffers, bind
//! groups, etc.
//! 
//! Resources are created via appropriate functions in the [`crate::render::api::Renderer`].
//! 

mod camera;
pub use camera::*;
mod shader;
pub use shader::*;
mod texture;
pub use texture::*;
mod skybox;
pub use skybox::*;
mod model;
pub use model::*;
