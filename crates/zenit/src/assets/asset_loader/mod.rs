use super::ZENIT_BUILTIN_LVL;
use crate::{assets::texture_loader::load_texture_as_asset, scene::EngineBorrow};
use log::*;
use std::io::{Cursor, Read, Seek};
use zenit_lvl::node::{read_node_children, read_node_header};
use zenit_utils::{ok, AnyResult};

pub mod shader_loader;
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
            // For clarity, keep the loader invocations as one-liners.
            match child.name.as_bytes() {
                b"tex_" => load_texture_as_asset((&mut r, child), self.engine),
                _ => {}
            }
        }
        ok()
    }

    pub fn load_builtins(&mut self) -> AnyResult {
        self.load_level_file(Cursor::new(ZENIT_BUILTIN_LVL), "zenit_builtin")?;
        self.engine.assets.error_texture = self
            .engine
            .assets
            .textures
            .get("zenit_error")
            .expect("zenit_error texture not found")
            .clone();
        ok()
    }
}
