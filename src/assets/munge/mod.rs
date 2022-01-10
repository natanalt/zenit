use std::io::{self, Read, Seek, SeekFrom};

mod name;
mod parser;

pub use name::MungeName;

#[derive(Debug, Clone)]
pub struct MungeNode {
    pub name: MungeName,
    pub offset: u64,
    pub length: u32,
}

impl MungeNode {
    /// Reads the internal node data into a `Box<[u8]>` from given `Read + Seek` reader.
    pub fn read_data<Reader: Read + Seek>(&self, r: &Reader) -> io::Result<Box<[u8]>> {
        let length = self.length as usize;
        let result = Vec::with_capacity(length as usize);
        result.resize(length, 0);

        r.seek(SeekFrom::Start(self.offset))?;
        r.read(&mut result)?;

        Ok(result.into_boxed_slice())
    }
}
