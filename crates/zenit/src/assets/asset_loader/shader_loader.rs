use std::{io::{Read, Seek, Cursor}, borrow::Cow};
use thiserror::Error;
use wgpu::{ShaderModuleDescriptor, ShaderSource};
use zenit_lvl::{node::{NodeHeader, NodeRead, read_node_children, read_node_header}, zext::LevelWgslShader};
use zenit_utils::{AnyResult, ok};
use crate::{graphics::{ShaderResource, Renderer}, assets::ZENIT_BUILTIN_LVL};
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

pub fn load_shader(
    (mut r, node): (impl Read + Seek, NodeHeader),
    renderer: &mut Renderer,
) -> Result<(String, ShaderResource), ShaderLoadError> {
    use ShaderLoadError::*;

    let node = LevelWgslShader::read_node_at(&mut r, node).map_err(|e| ParseError(e))?;
    let name = node.name.into_string().map_err(|_| BadName)?;
    let code = node.code.into_string().map_err(|_| BadCode)?;

    let module = renderer.dc.device.create_shader_module(ShaderModuleDescriptor {
        label: Some(&name),
        source: ShaderSource::Wgsl(Cow::Borrowed(&code)),
    });


    Ok((name.clone(), ShaderResource { name, code, module }))
}

pub fn load_builtin_shaders(renderer: &mut Renderer) -> AnyResult {
    let mut reader = Cursor::new(ZENIT_BUILTIN_LVL);
    let header = read_node_header(&mut reader)?;
    let children = read_node_children(&mut reader, header)?;

    for child in children {
        if child.name.as_bytes() == b"WGSL" {
            let (name, shader) = load_shader((&mut reader, child), renderer)?;
            trace!("Loaded shader `{name}`");
            renderer.shaders.insert(name, shader);
        }
    }

    ok()
}
