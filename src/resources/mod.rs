use crate::{
    engine::Engine,
    munge::{self, NodeName, TreeNode},
    unwrap_or_bail, unwrap_or_return_err, AnyResult,
};
use log::info;
use memmap2::Mmap;
use std::{
    collections::HashMap,
    ffi::CStr,
    fs::File,
    path::{Path, PathBuf},
};

pub mod script;
pub use script::ScriptResource;

pub enum Resource {
    Script(ScriptResource),
}

/// Namespace is the game's data manager
pub struct Resources {
    pub game_root: PathBuf,
    pub resources: HashMap<String, Resource>,
}

impl Resources {
    pub fn new<P: AsRef<Path>>(game_root: P) -> Self {
        Self {
            game_root: game_root.as_ref().to_path_buf(),
            resources: HashMap::new(),
        }
    }

    /// Unloads all loaded data
    pub fn unload_all(&mut self) {
        self.resources.clear();
    }

    pub fn resource_exists(&self, name: &str) -> bool {
        self.resources.contains_key(name)
    }

    /// Loads a single entire file into memory
    pub fn load_file<P: AsRef<Path>>(&mut self, path: P) -> AnyResult<()> {
        info!("Loading {}...", path.as_ref().display());

        let mut results = HashMap::new();

        let file = File::open(self.game_root.join(path))?;
        let memmap = unsafe { Mmap::map(&file)? };

        let nodes = munge::parse(&memmap)?.parse_children()?;
        for node in nodes {
            if let Some((name, resource)) = self.parse_resource(&node)? {
                results.insert(name, resource);
            }
        }

        for (name, value) in results.drain() {
            self.resources.insert(name, value);
        }

        Ok(())
    }

    /// Loads specified sections from a .lvl file's internal lvl_ nodes.
    pub fn load_file_sections<T: IntoIterator<Item = String>>(
        &mut self,
        _path: &str,
        _sections: T,
    ) -> AnyResult<()> {
        todo!()
    }

    /// Parses a given resource. Returns None if it's already loaded, or unsupported
    pub fn parse_resource(&mut self, node: &TreeNode) -> AnyResult<Option<(String, Resource)>> {

        Ok(match node.name() {
            ids::SCR_ => {
                let children = node.parse_children()?;
                let mut name = None;
                let mut info = None;
                let mut body = None;

                for child in &children {
                    match child.name() {
                        ids::NAME => {
                            name = CStr::from_bytes_with_nul(child.data())?
                                .to_str()?
                                .to_string()
                                .into();
                        }
                        ids::INFO => {
                            info = Some(child.data()[0]);
                        }
                        ids::BODY => {
                            body = Some(child.data());
                        }
                        _ => {}
                    }
                }

                let name = unwrap_or_bail!(name, "Name not present in `scr_` resource");

                log::debug!("Found script `{}`", &name);

                Some((name.clone(), Resource::Script(ScriptResource {
                    name,
                    info: unwrap_or_bail!(info, "Info not present in `scr_` resource"),
                    chunk: Vec::from(unwrap_or_bail!(body, "Chunk not present in `scr_` resource")),
                })))
            }
            _ => None,
        })
    }
}

mod ids {
    use crate::munge::NodeName;

    pub const SCR_: NodeName = NodeName::from_literal("scr_");
    pub const NAME: NodeName = NodeName::from_literal("NAME");
    pub const INFO: NodeName = NodeName::from_literal("INFO");
    pub const BODY: NodeName = NodeName::from_literal("BODY");
}
