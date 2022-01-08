use crate::{munge, AnyResult};
use log::info;
use memmap2::Mmap;
use std::{
    collections::HashMap,
    fs::File,
    path::{Path, PathBuf},
};

pub mod parser;
pub mod script;
pub use script::ScriptResource;

pub enum Resource {
    Script(ScriptResource),
}

/// Namespace is the game's data manager
pub struct Resources {
    pub game_root: PathBuf,
    pub persistent: HashMap<String, Resource>,
    pub temporary: HashMap<String, Resource>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerKind {
    /// Persistent resources aren't unloaded until the whole game shuts down
    Persistent,
    /// Temporary resources are unloaded between loading screens
    Temporary,
}

impl Resources {
    pub fn new<P: AsRef<Path>>(game_root: P) -> Self {
        Self {
            game_root: game_root.as_ref().to_path_buf(),
            persistent: HashMap::new(),
            temporary: HashMap::new(),
        }
    }

    /// Creates a path within the game's main level data directory
    pub fn data_path<P: AsRef<Path>>(&self, relative: P) -> PathBuf {
        self.game_root.join("GameData/data/_lvl_pc").join(relative)
    }

    /// Checks whether the resource exists in temporary or persistent storage
    pub fn resource_exists(&self, name: &str) -> bool {
        self.temporary.contains_key(name) || self.persistent.contains_key(name)
    }

    /// Unloads all temporary loaded data
    pub fn unload_all(&mut self) {
        self.temporary.clear();
    }

    pub fn get_container(&mut self, target: ContainerKind) -> &mut HashMap<String, Resource> {
        match target {
            ContainerKind::Persistent => &mut self.persistent,
            ContainerKind::Temporary => &mut self.temporary,
        }
    }
}

impl Resources {
    /// Loads a single entire file into memory
    pub fn load_file<P: AsRef<Path>>(&mut self, path: P, target: ContainerKind) -> AnyResult<()> {
        info!(
            "Loading `{}`...",
            path.as_ref()
                .strip_prefix(&self.game_root)
                .expect("Invalid load path (not game root relative)")
                .display()
        );

        let file = File::open(self.game_root.join(path))?;
        let memmap = unsafe { Mmap::map(&file)? };

        for node in munge::parse(&memmap)?.parse_children()? {
            if let Some((name, resource)) = self.parse_resource(&node)? {
                self.get_container(target).insert(name, resource);
            }
        }

        Ok(())
    }
}
