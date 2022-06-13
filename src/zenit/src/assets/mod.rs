use std::collections::HashMap;
use zenit_utils::AnyResult;
use crate::root::GameRoot;

pub mod texture;

/// Manages metadata for all BF2 objects (textures, models, props, worlds, etc.)
pub struct AssetLoader {
    indexes: HashMap<String, ResourceIndex>,
}

impl AssetLoader {
    pub fn new() -> Self {
        Self {
            indexes: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.indexes.clear();
    }

    pub fn open_level_file(&mut self, _root: &GameRoot, _path: String) -> AnyResult {
        todo!()
    }
}

/// Represents an index of resources
pub struct ResourceIndex {
}
