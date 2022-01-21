use super::munge::{parser::ParseError, MungeName, MungeNode, MungeTreeNode};
use crate::{
    args::ZenitArgs,
    assets::{loader, AssetKind},
    utils::fnv1a_hash,
    AnyResult,
};
use bevy::{
    asset::{AssetIo, AssetIoError, BoxedFuture},
    prelude::*,
    tasks::IoTaskPool,
    utils::HashMap,
};
use dashmap::DashMap;
use std::{
    ffi::CStr,
    fs::{self, File},
    io::{self, BufReader, Read},
    path::{Component, Path, PathBuf},
    sync::{Arc, RwLock},
};
use thiserror::Error;

pub struct MungeAssetIoPlugin;

impl Plugin for MungeAssetIoPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AssetServer::new(
            MungeAssetIo::new(
                app.world
                    .get_resource::<ZenitArgs>()
                    .expect("Zenit args resource not found")
                    .game_root
                    .clone(),
            ),
            app.world
                .get_resource::<IoTaskPool>()
                .expect("`IoTaskPool` resource not found.")
                .0
                .clone(),
        ));
    }
}

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
    cache: DashMap<PathBuf, Arc<RwLock<MungeCache>>, ahash::RandomState>,
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

impl From<CacheOpenError> for io::Error {
    fn from(error: CacheOpenError) -> io::Error {
        match error {
            CacheOpenError::IoError(err) => err,
            _ => io::ErrorKind::InvalidData.into(),
        }
    }
}

impl MungeAssetIo {
    pub fn new<P: Into<PathBuf>>(root_path: P) -> Self {
        Self {
            root_path: root_path.into(),
            cache: DashMap::default(),
        }
    }

    /// Creates a new cache entry and returns it, or returns the existing one. May fail for IO
    /// or file format reasons.
    fn open<P: Into<PathBuf>>(
        &self,
        munge_file_path: P,
    ) -> Result<Arc<RwLock<MungeCache>>, CacheOpenError> {
        let file_path = munge_file_path.into();
        if !self.cache.contains_key(&file_path) {
            let cache_object = MungeCache::open(&file_path)?;
            self.cache
                .insert(file_path.clone(), Arc::new(RwLock::new(cache_object)));
        }
        Ok(self.cache.get(&file_path).unwrap().clone())
    }
}

impl AssetIo for MungeAssetIo {
    fn load_path<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<'a, AnyResult<Vec<u8>, AssetIoError>> {
        info!("Loading `{:?}`", path);
        Box::pin(async move {
            if let Some((file_path, munge_path)) = MungePath::parse_path(path) {
                let full_file_path = self.root_path.join(&file_path);
                let cache_arc = self
                    .open(&full_file_path)
                    .map_err(|err| io::Error::from(err))?;
                let mut cache = cache_arc.write().unwrap();

                match munge_path {
                    MungePath::RootObject(name) => Ok(cache
                        .root_assets
                        .iter()
                        .find(|asset| asset.make_filename() == name)
                        .ok_or_else(|| AssetIoError::NotFound(path.to_owned()))?
                        .node
                        .clone()
                        .read_with_header(&mut cache.file)?),
                    MungePath::SectionObject {
                        section_name,
                        object_name,
                    } => Ok(cache
                        .sections
                        .get(&fnv1a_hash(section_name.as_bytes()))
                        .ok_or_else(|| AssetIoError::NotFound(path.to_owned()))?
                        .iter()
                        .find(|asset| asset.make_filename() == object_name)
                        .ok_or_else(|| AssetIoError::NotFound(path.to_owned()))?
                        .node
                        .clone()
                        .read_with_header(&mut cache.file)?),
                    _ => Err(io::Error::from(io::ErrorKind::InvalidInput).into()),
                }
            } else {
                let mut bytes = Vec::new();
                let full_path = self.root_path.join(path);
                match File::open(&full_path) {
                    Ok(mut file) => {
                        file.read_to_end(&mut bytes)?;
                    }
                    Err(e) => {
                        return if e.kind() == std::io::ErrorKind::NotFound {
                            Err(AssetIoError::NotFound(full_path))
                        } else {
                            Err(e.into())
                        }
                    }
                }
                Ok(bytes)
            }
        })
    }

    fn read_directory(
        &self,
        path: &Path,
    ) -> AnyResult<Box<dyn Iterator<Item = PathBuf>>, AssetIoError> {
        if let Some((file_path, munge_path)) = MungePath::parse_path(path) {
            let cache_arc = self.open(&file_path).map_err(|err| io::Error::from(err))?;
            let cache = cache_arc.read().expect("lock poisoned");

            match munge_path {
                MungePath::RootDirectory => Ok(Box::new(
                    cache.root_assets.clone().into_iter().map(move |asset| {
                        let mut result = PathBuf::new();
                        result.push(file_path.clone());
                        result.push(asset.make_filename());
                        result
                    }),
                )),
                MungePath::SectionDirectory(name) => {
                    let hash = fnv1a_hash(name.as_bytes());
                    let assets = cache
                        .sections
                        .get(&hash)
                        .ok_or_else(|| AssetIoError::NotFound(path.to_owned()))?;

                    Ok(Box::new(assets.clone().into_iter().map(move |asset| {
                        let mut result = PathBuf::new();
                        result.push(file_path.clone());
                        result.push(format!("${}", name));
                        result.push(asset.make_filename());
                        result
                    })))
                }
                _ => Err(io::Error::from(io::ErrorKind::InvalidInput).into()),
            }
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

/// Representation of a single cached munge file. Contains information about root assets and assets
/// within sections.
#[derive(Debug)]
struct MungeCache {
    pub file: BufReader<File>,
    //pub root_node: MungeTreeNode,
    pub root_assets: Vec<AssetNode>,
    pub sections: HashMap<u32, Vec<AssetNode>>,
}

/// Represents an asset node and its cached name.
#[derive(Debug, Clone)]
struct AssetNode {
    pub node: MungeNode,
    pub name: String,
}

impl AssetNode {
    pub fn make_filename(&self) -> String {
        format!(
            "{}.{}",
            &self.name,
            AssetKind::from_node_name(self.node.name).extension()
        )
    }
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

        let mut root_node = MungeTreeNode::parse(&mut file, Some(2)).map_err(|err| match err {
            ParseError::InvalidNode => CacheOpenError::InvalidRootNode,
            ParseError::IoError(err) => CacheOpenError::IoError(err),
        })?;

        let mut root_children = Vec::new();
        let mut sections = HashMap::default();

        let parse_asset = |file: &mut BufReader<File>, child: &MungeTreeNode| {
            let name_bytes = child
                .children
                .iter()
                .find(|x| x.node.name == MungeName::from_literal("NAME"))
                .ok_or(CacheOpenError::InvalidChildNode)?
                .node
                .read_contents(file)?;

            let name = CStr::from_bytes_with_nul(&name_bytes)
                .map_err(|_| CacheOpenError::InvalidChildNode)?
                .to_str()
                .map_err(|_| CacheOpenError::InvalidChildNode)?
                .to_string();

            Result::<AssetNode, CacheOpenError>::Ok(AssetNode {
                node: child.node,
                name,
            })
        };

        for child in &mut root_node.children {
            if child.node.name == MungeName::from_literal("lvl_") {
                let section_root = child
                    .children
                    .first_mut()
                    .ok_or(CacheOpenError::InvalidChildNode)?;

                section_root.parse_tree(&mut file, Some(2))?;

                let node_hash: u32 = section_root.node.name.into();
                let mut assets = Vec::new();
                for child in &section_root.children {
                    assets.push(parse_asset(&mut file, child)?);
                }

                sections.insert(node_hash, assets);
            } else if loader::is_loadable(child.node.name) {
                root_children.push(parse_asset(&mut file, child)?);
            }
        }

        Ok(MungeCache {
            file,
            //root_node,
            root_assets: root_children,
            sections,
        })
    }
}
