use super::ZENIT_BUILTIN_LVL;
use crate::{assets::asset_loader::texture_loader::LoadedTexture, scene::EngineBorrow};
use log::*;
use std::io::{Cursor, Read, Seek};
use texture_loader::load_texture;
use zenit_lvl::node::{read_node_children, read_node_header};
use zenit_utils::{ok, AnyResult};

pub mod texture_loader;

pub struct AssetLoader<'a> {
    engine: &'a mut EngineBorrow<'a>,
}

impl<'a> AssetLoader<'a> {
    pub fn new(engine: &'a mut EngineBorrow<'a>) -> Self {
        Self { engine }
    }

    pub fn load_level_file(&mut self, mut r: impl Read + Seek, label: &str) -> AnyResult {
        trace!("Loading level file `{label}`...");

        let header = read_node_header(&mut r)?;
        let children = read_node_children(&mut r, header)?;

        for child in children {
            match child.name.as_bytes() {
                // TODO: move this entire match arm into texture_loader.rs
                b"tex_" => {
                    let (name, texture) = match load_texture(&mut r, child, self.engine) {
                        Ok(v) => v,
                        Err(e) => {
                            error!("An error occurred while loading a texture: {e:#?}");
                            continue;
                        }
                    };

                    match texture {
                        LoadedTexture::Texture(texture) => {
                            trace!("Loaded texture `{name}`...");
                            self.engine.assets.textures.insert(name, texture);
                        }
                        LoadedTexture::Cubemap(cubemap) => {
                            trace!("Loaded cubemap `{name}`...");
                            self.engine.assets.cubemaps.insert(name, cubemap);
                        }
                    }
                }
                _ => {}
            }
        }
        ok()
    }

    pub fn load_builtins(&mut self) -> AnyResult {
        self.load_level_file(Cursor::new(ZENIT_BUILTIN_LVL), "zenit_builtin")?;
        ok()
    }
}
