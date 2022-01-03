use crate::AnyResult;

pub enum Resource {
}

/// Namespace is the game's data manager
pub struct Namespace {
    resources: Vec<Resource>,
}

impl Namespace {
    pub fn new() -> Self {
        Self {
            resources: Vec::new(),
        }
    }

    /// Unloads all loaded data
    pub fn unload_all(&mut self) {
        self.resources.clear();
        todo!()    
    }

    /// Loads a single entire file into memory
    pub fn load_file(&mut self, _path: &str) -> AnyResult<()> {
        todo!()
    }

    /// Loads specified sections from a .lvl file's internal lvl_ nodes.
    pub fn load_file_sections<T: Into<String>>(
        &mut self,
        _path: &str,
        _sections: &[T],
    ) -> AnyResult<()> {
        todo!()
    }
}
