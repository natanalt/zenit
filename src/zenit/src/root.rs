use std::{path::{PathBuf, Path}, io, fs::{File, self}};
use log::info;

// oh yes
const INVALID_MARKER: &str = "/this/is/an/invalid/path/please/do/not/use";

#[derive(Clone)]
pub struct GameRoot {
    pub path: PathBuf,
}

impl GameRoot {
    pub fn new(override_path: Option<&PathBuf>) -> Self {
        let result = if let Some(path) = override_path.map(PathBuf::clone) {
            Self { path }
        } else if let Some(path) = crate::platform::find_bf2() {
            Self { path }
        } else {
            Self { path: PathBuf::from(INVALID_MARKER) }
        };

        if !result.is_invalid() {
            info!("Using game root: {}", result.path.display());
        }

        result
    }

    pub fn is_invalid(&self) -> bool {
        self.path.to_str() == Some(INVALID_MARKER)
    }

    #[inline]
    pub fn make_path(&self, relative: impl AsRef<Path>) -> PathBuf {
        self.path.join(relative).canonicalize().unwrap()
    }

    #[inline]
    pub fn open_read_file(&self, relative: impl AsRef<Path>) -> io::Result<File> {
        File::open(self.make_path(relative))
    }

    #[inline]
    pub fn read_file(&self, relative: impl AsRef<Path>) -> io::Result<Vec<u8>> {
        fs::read(self.make_path(relative))
    }
}
