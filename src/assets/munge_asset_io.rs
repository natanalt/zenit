use super::munge::{
    parser::{ParseChildrenError, ParseError},
    MungeName, MungeNode,
};
use crate::{
    assets::{loader, munge},
    AnyResult, utils,
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
        Box::pin(async move {
            let (file_path, inner_path) = MungePath::parse_path(path)
                .ok_or_else(|| AssetIoError::NotFound(path.to_owned()))?;

            if inner_path.is_directory() {
                return Err(AssetIoError::Io(io::ErrorKind::InvalidInput.into()));
            }

            // There's a little tiny chance that I do blocking file I/O in an async context...
            // That's really unfortunate
            // TODO: make node loading async-friendly
            let cache_arc = self.open(file_path).map_err(|err| match err {
                CacheOpenError::IoError(e) => e,
                _ => io::ErrorKind::InvalidData.into(),
            })?;

            let mut cache = cache_arc.lock().expect("cache lock poisoned");

            match inner_path {
                MungePath::RootObject(name) => {
                    let child = cache
                        .cached_children
                        .iter()
                        .find(|child| child.name == name)
                        .ok_or_else(|| AssetIoError::NotFound(path.to_owned()))?
                        .node
                        .clone();
                    
                    Ok(child.read_data(&mut cache.file)?)
                }
                MungePath::SectionObject {
                    section_name,
                    object_name,
                } => {
                    let hash = utils::hash(section_name.as_bytes());
                    todo!()
                },
                _ => unreachable!(),
            }
        })
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
            .unwrap_or(false)
            || self.root_path.join(path).is_dir()
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
    pub root_node: MungeNode,
    pub cached_children: Vec<LoadableCacheNode>,
}

#[derive(Debug)]
struct LoadableCacheNode {
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
        let root_node = munge::parse(&mut file).map_err(|err| match err {
            ParseError::InvalidNode => CacheOpenError::InvalidFormat,
            ParseError::IoError(e) => e.into(),
        })?;

        let children = root_node
            .parse_children(&mut file)
            .map_err(|err| match err {
                ParseChildrenError::NoChildren => CacheOpenError::InvalidRootNode,
                ParseChildrenError::IoError(e) => e.into(),
            })?;

        let mut cached_children = Vec::new();
        for node in children {
            if !loader::is_loadable(node.name) {
                continue;
            }

            let children = node.parse_children(&mut file).map_err(|err| match err {
                ParseChildrenError::NoChildren => CacheOpenError::InvalidChildNode,
                ParseChildrenError::IoError(e) => e.into(),
            })?;

            let name = std::ffi::CString::from_vec_with_nul(
                children
                    .into_iter()
                    .find(|child| child.name == MungeName::from_literal("NAME"))
                    .ok_or(CacheOpenError::InvalidChildNode)?
                    .read_data(&mut file)?,
            )
            .map_err(|_| CacheOpenError::InvalidChildNode)?
            .into_string()
            .map_err(|_| CacheOpenError::InvalidChildNode)?;

            cached_children.push(LoadableCacheNode { node, name });
        }

        Ok(MungeCache {
            file,
            root_node,
            cached_children,
        })
    }
}
