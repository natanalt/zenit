use crate::AnyResult;
use bevy::{
    asset::{AssetIo, AssetIoError, BoxedFuture},
    prelude::*,
};
use std::{path::{Path, PathBuf}, fs};

/// Asset IO with a special case handling for munge (.lvl) files. Otherwise it behaves like a custom
/// file asset IO, although without change watch support.
///
/// ## Example paths
///  * `data/shell.lvl` - level file, treated like a directory
///  * `data/shell.lvl/shell_interface.luac` - single `scr_` component called `shell_interface`
///  * `side/all.lvl/$/all_fly_snowspeeder` - directory-like data from name-hashed `lvl_` node (note the $)
///  * `side/all.lvl/$/all_fly_snowspeeder/snowspeeder_icon.ztexture` - single texture from the `lvl_` node
/// 
struct MungeAssetIo {
    root_path: PathBuf,
}

impl AssetIo for MungeAssetIo {
    fn load_path<'a>(
        &'a self,
        path: &'a Path,
    ) -> BoxedFuture<'a, AnyResult<Vec<u8>, AssetIoError>> {
        todo!()
    }

    fn read_directory(
        &self,
        path: &Path,
    ) -> AnyResult<Box<dyn Iterator<Item = PathBuf>>, AssetIoError> {
        if let Some((file_path, munge_path)) = self.lvl_path_parse(path) {
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
        self.lvl_path_parse(path).is_some() || self.root_path.join(path).is_dir()
    }

    fn watch_path_for_changes(&self, _path: &Path) -> AnyResult<(), AssetIoError> {
        Ok(())
    }

    fn watch_for_changes(&self) -> AnyResult<(), AssetIoError> {
        Ok(())
    }
}

impl MungeAssetIo {
    fn lvl_path_parse(&self, path: &Path) -> Option<(PathBuf, MungePath)> {
        todo!()
    }
}

enum MungePath {
    /// Example: `data/shell.lvl`
    RootDirectory,
    /// Example: `data/shell.lvl/shell_interface.luac`
    RootObject(String),
    /// Example: `side/all.lvl/$/all_fly_snowspeeder`
    SectionDirectory(String),
    /// Example: `side/all.lvl/$/all_fly_snowspeeder/snowspeeder_icon.ztexture`
    SectionObject{ section_name: String, object_name: String },
}
