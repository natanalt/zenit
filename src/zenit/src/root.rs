use std::{path::{PathBuf, Path}, io, fs::{File, self}};

#[derive(Clone)]
pub struct GameRoot {
    pub path: PathBuf,
}

impl GameRoot {
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
