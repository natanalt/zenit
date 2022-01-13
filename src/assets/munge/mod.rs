use std::io::{self, Read, Seek, SeekFrom};

mod name;
pub mod parser;

pub use name::MungeName;

#[derive(Debug, Clone, Copy)]
pub struct MungeNode {
    pub name: MungeName,
    pub offset: u64,
    pub length: u32,
}

impl MungeNode {
    /// Reads the internal node data into a `Box<[u8]>` from given `Read + Seek` reader.
    pub fn read_data<Reader: Read + Seek>(&self, mut r: Reader) -> io::Result<Vec<u8>> {
        let length = self.length as usize;
        let mut result = Vec::with_capacity(length);
        result.resize(length, 0);

        r.seek(SeekFrom::Start(self.offset))?;
        r.read(&mut result)?;

        Ok(result)
    }
}

pub fn parse<Reader: Read + Seek>(r: &mut Reader) -> Result<MungeNode, parser::ParseError> {
    MungeNode::parse(r)
}
