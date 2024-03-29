use log::info;
use std::{
    fs::{self, File},
    io,
    path::{Path, PathBuf},
};

/// A directory is considered "valid" if it hits all required files
const REQUIRED_FILES: &[&str] = &["common.lvl", "core.lvl", "ingame.lvl", "mission.lvl"];

/// Game root contains an internal path that points to the `_lvl_pc` directory
/// of the game.
#[derive(Clone, Default)]
pub struct GameRoot {
    /// Directory with the main game contents.
    pub main: PathBuf,
    // Someday...
    //pub addons: PathBuf,
}

impl GameRoot {
    /// Given the override path, attempts to construct a GameRoot instance.
    /// A direct path to `_lvl_pc` is not necessary, the code will attempt
    /// to scan up to 4 levels deep looking for a valid data directory.
    pub fn new(override_path: Option<&PathBuf>) -> Self {
        let result = if let Some(path) = override_path.map(PathBuf::clone) {
            Self {
                main: scan_directory_for_main(path, 0)
                    .ok()
                    .flatten()
                    .unwrap_or_else(|| PathBuf::new()),
            }
        } else if let Some(path) = crate::platform::find_bf2() {
            Self { main: path }
        } else {
            Self {
                main: PathBuf::new(),
            }
        };
        if result.main.as_os_str().len() != 0 {
            info!("Using game root: {}", result.main.display());
        }

        result
    }

    pub fn make_path(&self, relative: impl AsRef<Path>) -> PathBuf {
        self.main.join(relative).canonicalize().unwrap()
    }

    pub fn open_read_file(&self, relative: impl AsRef<Path>) -> io::Result<File> {
        File::open(self.make_path(relative))
    }

    pub fn read_file(&self, relative: impl AsRef<Path>) -> io::Result<Vec<u8>> {
        fs::read(self.make_path(relative))
    }
}

/// Attempts to scan this directory for a valid main directory. If none is
/// found, or depth limit is exceeded, returns None.
fn scan_directory_for_main(root: PathBuf, depth: u32) -> io::Result<Option<PathBuf>> {
    const MAX_SCAN_DEPTH: u32 = 4;

    if depth > MAX_SCAN_DEPTH {
        return Ok(None);
    }

    let mut required_count = 0;
    let mut dirs = vec![];
    let entries = fs::read_dir(&root)?.take(50).collect::<Vec<_>>();
    for entry in entries {
        let entry = entry?;
        let ftype = entry.file_type()?;
        if ftype.is_file() {
            if let Some(name) = entry.file_name().to_str() {
                if REQUIRED_FILES.contains(&name) {
                    required_count += 1;
                }
            }
        } else if ftype.is_dir() {
            dirs.push(entry);
        }
    }

    if required_count == REQUIRED_FILES.len() {
        Ok(Some(root))
    } else {
        for dir in dirs {
            if let Some(path) = scan_directory_for_main(dir.path(), depth + 1)? {
                return Ok(Some(path));
            }
        }
        Ok(None)
    }
}
