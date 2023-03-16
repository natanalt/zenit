use super::{root::GameRoot, texture::load_level_texture};
use crate::render::{Renderer, TextureHandle};
use ahash::AHashMap;
use log::*;
use std::{
    ffi::CStr,
    fs::{File, OpenOptions},
    sync::Arc,
};
use zenit_lvl::{game::LevelData, FromNode};
use zenit_utils::{ok, AnyResult, AsciiDisplay};

/// The asset manager is responsible for caching and loading various game assets.
pub struct AssetManager {
    pub game_root: GameRoot,
    pub open_files: AHashMap<String, CachedDataFile>,
    pub textures: AHashMap<String, TextureHandle>,

    pub error_texture: TextureHandle,
}

impl AssetManager {
    pub fn new(game_root: GameRoot) -> Self {
        todo!()
    }

    #[inline]
    pub fn ensure_file_opened(&mut self, path: &str) -> AnyResult {
        if !self.open_files.contains_key(path) {
            self.open_files
                .insert(path.to_string(), CachedDataFile::open(path)?);
        }
        ok()
    }

    /// Opens a data file and performs basic parsing on it, without properly loading any assets.
    pub fn open_file(&mut self, path: &str) -> AnyResult<&CachedDataFile> {
        // Rust's borrow checker model doesn't let me just do an `if let` >:c
        // I thought non-lexical lifetimes were meant to fix that >:c >:c
        if self.open_files.contains_key(path) {
            return Ok(self.open_files.get(path).unwrap());
        }

        self.ensure_file_opened(path)?;
        Ok(self.open_files.get(path).unwrap())
    }

    /// Loads all non-pack assets from a file.
    pub fn load_file(&mut self, renderer: &mut Renderer, path: &str) -> AnyResult {
        self.ensure_file_opened(path)?;
        let file = self.open_files.get(path).unwrap();
        let data = &file.data;

        for texture in &data.textures {
            let Some(name) = validate_name(path, &texture.name) else { continue };

            self.textures
                .insert(name, load_level_texture(renderer, texture)?);
        }

        ok()
    }

    pub fn get_texture(&mut self, name: &str) -> TextureHandle {
        if let Some(texture) = self.textures.get(name) {
            texture.clone()
        } else {
            warn!("Texture `{name}` not found");
            self.error_texture.clone()
        }
    }

    pub fn unload_resources(&mut self) {
        self.textures.clear();
    }

    pub fn close_open_files(&mut self) {
        self.open_files.clear();
    }
}

pub struct AssetBundle {
    name: String,
    textures: AHashMap<String, TextureHandle>,
}

pub struct CachedDataFile {
    pub handle: File,
    pub data: LevelData,
}

impl CachedDataFile {
    pub fn open(path: &str) -> AnyResult<Self> {
        let mut options = OpenOptions::new();
        options.read(true);
        options.write(false);

        #[cfg(target_os = "windows")]
        {
            // On Windows, set up the share flags so that other processes can open the file,
            // but not modify it.
            use winapi::um::winnt::FILE_SHARE_READ;
            use std::os::windows::fs::OpenOptionsExt;
            options.share_mode(FILE_SHARE_READ);
        }

        let mut handle = options.open(path)?;
        let data = LevelData::from_read(&mut handle)?;

        Ok(Self { handle, data })
    }
}

fn validate_name(path: &str, name: &CStr) -> Option<String> {
    match name.to_owned().into_string() {
        Ok(ok_name) => Some(ok_name),
        Err(_) => {
            let bad_name = AsciiDisplay(name.to_bytes());
            warn!("Invalid texture name in file `{path}` - `{bad_name}`");
            None
        }
    }
}
