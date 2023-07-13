use std::{io::{Read, Seek}, borrow::Cow};
use thiserror::Error;
use wgpu::{ShaderModuleDescriptor, ShaderSource};
use zenit_lvl::{node::{NodeHeader, NodeRead}, zext::LevelWgslShader};
use crate::{graphics::ShaderResource, scene::EngineBorrow};
use log::*;

#[derive(Debug, Error)]
pub enum ShaderLoadError {
    #[error("the shader has an invalid name")]
    BadName,
    #[error("the shader has an improperly formatted code")]
    BadCode,
    #[error("a node parsing error occurred: {0:#?}")]
    ParseError(anyhow::Error),
}

pub fn load_shader_as_asset(
    (mut r, node): (impl Read + Seek, NodeHeader),
    engine: &mut EngineBorrow,
) {
    match load_shader((&mut r, node), engine) {
        Ok((name, shader)) => {
            trace!("Loaded shader `{name}`...");
            engine.renderer.shaders.insert(name, shader);
        }
        Err(e) => {
            error!("An error occurred while loading a shader: {e:#?}");
        }
    };
}

pub fn load_shader(
    (mut r, node): (impl Read + Seek, NodeHeader),
    engine: &mut EngineBorrow,
) -> Result<(String, ShaderResource), ShaderLoadError> {
    use ShaderLoadError::*;

    let node = LevelWgslShader::read_node_at(&mut r, node).map_err(|e| ParseError(e))?;
    let name = node.name.into_string().map_err(|_| BadName)?;
    let code = node.code.into_string().map_err(|_| BadCode)?;

    let module = engine.renderer.dc().device.create_shader_module(ShaderModuleDescriptor {
        label: Some(&name),
        source: ShaderSource::Wgsl(Cow::Borrowed(&code)),
    });

    Ok((name.clone(), ShaderResource { name, code, module }))
}
