use super::munge::{parser::ParseError, MungeName, MungeNode, MungeTreeNode};
use crate::{
    assets::{loader, munge},
    utils, AnyResult,
};
use bevy::{
    asset::{AssetIo, AssetIoError, BoxedFuture},
    prelude::*,
    utils::HashMap,
};
use std::{
    fs::{self, File},
    io::{self, BufReader, Seek, SeekFrom},
    path::{Component, Path, PathBuf},
    sync::{Arc, Mutex},
};
use thiserror::Error;

/// Asset IO with a special case handling for munge (.lvl) files. Otherwise it behaves like a custom
/// file asset IO, although without change watch support.
///
/// ## Example paths
///  * `data/shell.lvl` - level file, treated like a directory (won't read section names)
///  * `data/shell.lvl/shell_interface.luac` - single `scr_` component called `shell_interface`
///  * `side/all.lvl/$all_fly_snowspeeder` - directory-like data from name-hashed `lvl_` node (note the $)
///  * `side/all.lvl/$all_fly_snowspeeder/snowspeeder_icon.ztexture` - single texture from the `lvl_` node
///
pub struct MungeAssetIo {
    root_path: PathBuf,
    // TODO: use something like dashmap for a faster concurrent HashMap
    cache: Mutex<HashMap<PathBuf, Arc<Mutex<MungeCache>>>>,
}

#[derive(Debug, Error)]
pub enum CacheOpenError {
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    #[error("The file is not a valid munge file")]
    InvalidFormat,
    #[error("The root node does not seem to contain nodes")]
    InvalidRootNode,
    #[error("A child node wasn't in a valid format")]
    InvalidChildNode,
}

impl MungeAssetIo {
    pub fn new<P: Into<PathBuf>>(root_path: P) -> Self {
        Self {
            root_path: root_path.into(),
            cache: Mutex::new(HashMap::default()),
        }
    }

    /// Creates a new cache entry and returns it, or returns the existing one. May fail for IO
    /// or file format reasons. The munge file must contain root child nodes.
    fn open<P: Into<PathBuf>>(
        &self,
        munge_path: P,
    ) -> Result<Arc<Mutex<MungeCache>>, CacheOpenError> {
        let mut cache = self.cache.lock().expect("Posioned cache lock");

        let munge_path = munge_path.into();
        if !cache.contains_key(&munge_path) {
            let cache_object = MungeCache::open(&munge_path)?;
            cache.insert(munge_path.clone(), Arc::new(Mutex::new(cache_object)));
        }
        Ok(cache.get(&munge_path).unwrap().clone())
    }
}

impl AssetIo for MungeAssetIo {
    fn load_path<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<'a, AnyResult<Vec<u8>, AssetIoError>> {
        Box::pin(async move { todo!() })
    }

    fn read_directory(
        &self,
        path: &Path,
    ) -> AnyResult<Box<dyn Iterator<Item = PathBuf>>, AssetIoError> {
        if let Some((file_path, munge_path)) = MungePath::parse_path(path) {
            todo!()
        } else {
            let root_path = self.root_path.to_owned();
            Ok(Box::new(fs::read_dir(root_path.join(path))?.map(
                move |entry| {
                    let path = entry.unwrap().path();
                    path.strip_prefix(&root_path).unwrap().to_owned()
                },
            )))
        }
    }

    fn is_directory(&self, path: &Path) -> bool {
        MungePath::parse_path(path)
            .map(|f| f.1.is_directory())
            .unwrap_or_else(|| self.root_path.join(path).is_dir())
    }

    fn watch_path_for_changes(&self, _path: &Path) -> AnyResult<(), AssetIoError> {
        Ok(())
    }

    fn watch_for_changes(&self) -> AnyResult<(), AssetIoError> {
        Ok(())
    }
}

#[derive(Debug, Clone)]
enum MungePath {
    /// Example: `data/shell.lvl`
    RootDirectory,
    /// Example: `data/shell.lvl/shell_interface.luac`
    RootObject(String),
    /// Example: `side/all.lvl/$all_fly_snowspeeder`
    SectionDirectory(String),
    /// Example: `side/all.lvl/$all_fly_snowspeeder/snowspeeder_icon.ztexture`
    SectionObject {
        section_name: String,
        object_name: String,
    },
}

impl MungePath {
    pub fn parse_path(logical_path: &Path) -> Option<(PathBuf, MungePath)> {
        // Currently the algorithm just checks for a .lvl, as really the only case for a .lvl
        // extension is a level file. Just in case, we also verify that it's a file.

        let mut file_path = PathBuf::new();
        let mut path_kind: Option<MungePath> = None;

        for component in logical_path.components() {
            if let Some(kind) = path_kind {
                // At this point paths should be as simple as possible, no indirection
                let path_component = match component {
                    Component::Normal(x) => match x.to_str() {
                        Some(x) => x.to_string(),
                        None => return None,
                    },
                    _ => return None,
                };

                path_kind = Some(match kind {
                    MungePath::RootDirectory => {
                        if path_component.starts_with('$') {
                            MungePath::SectionDirectory(path_component[1..].to_string())
                        } else {
                            MungePath::RootObject(path_component)
                        }
                    }

                    MungePath::SectionDirectory(section_name) => MungePath::SectionObject {
                        section_name,
                        object_name: path_component,
                    },

                    // No further paths should be described at this point
                    MungePath::RootObject(_) | MungePath::SectionObject { .. } => return None,
                });
            } else {
                file_path.push(component);

                if let Component::Normal(path) = component {
                    let path_string = match path.to_str() {
                        Some(x) => x.to_lowercase(),
                        None => return None,
                    };

                    if path_string.ends_with(".lvl") {
                        //if !file_path.is_file() {
                        //    return None;
                        //}
                        path_kind = Some(MungePath::RootDirectory);
                    }
                }
            }
        }

        match path_kind {
            Some(path_kind) => (file_path, path_kind).into(),
            None => None,
        }
    }

    pub fn is_directory(&self) -> bool {
        match self {
            MungePath::RootDirectory | MungePath::SectionDirectory(_) => true,
            MungePath::RootObject(_) | MungePath::SectionObject { .. } => false,
        }
    }
}

#[derive(Debug)]
struct MungeCache {
    pub file: BufReader<File>,
    pub root_node: MungeTreeNode,
    pub root_children: HashMap,
    pub sections: HashMap<u32, Vec<AssetNode>>,
}

#[derive(Debug)]
struct AssetNode {
    pub node: MungeNode,
    pub name: String,
}

impl MungeCache {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<MungeCache, CacheOpenError> {
        let mut open_options = File::options();
        open_options.create(false).read(true).write(false);
        if cfg!(windows) {
            // On Windows, configure file sharing so that other processes can open and read
            // the file, but not modify it.
            // 1 = FILE_SHARE_READ
            use std::os::windows::fs::OpenOptionsExt;
            open_options.share_mode(1);
        }

        let mut file = BufReader::new(open_options.open(path)?);
        let root_node = MungeTreeNode::parse(&mut file, Some(2)).map_err(|err| match err {
            ParseError::InvalidNode => CacheOpenError::InvalidRootNode,
            ParseError::IoError(err) => CacheOpenError::IoError(err),
        })?;

        let root_children = root_node
            .children
            .iter()
            .


        todo!()
    }
}
