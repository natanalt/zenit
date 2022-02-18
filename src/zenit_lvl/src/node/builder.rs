use super::NodeName;
use anyhow::ensure;
use byteorder::{WriteBytesExt, LE};
use std::io::{self, Read, Seek, SeekFrom, Write};
use zenit_utils::AnyResult;

pub struct NodeBuilder<'w, W: Write + Seek> {
    /// Target writer
    w: &'w mut W,
    /// Stream position of this node's beginning, as in where its name starts
    origin: u64,
    /// Marks whether this node is meant to be node count prefixed
    prefixed: bool,
    /// Total size of this node's payload in bytes
    length: u32,
    /// Amount of written nodes so far (see `prefixed`)
    written_nodes: u32,
    /// Whether arbitrary raw data has been written to the node, making `prefixed` invalid
    arbitrary: bool,
}

impl<'w, W: Write + Seek> NodeBuilder<'w, W> {
    pub fn new(w: &'w mut W, name: NodeName) -> AnyResult<Self> {
        let origin = w.stream_position()?;
        w.write_u32::<LE>(name.into())?;
        w.write_u32::<LE>(0)?; // will be overwritten
        Ok(Self {
            w,
            origin,
            prefixed: false,
            length: 0,
            written_nodes: 0,
            arbitrary: false,
        })
    }

    /// Marks this node to include a node count prefix.
    pub fn count_prefixed(&mut self) -> AnyResult<&mut Self> {
        ensure!(self.length == 0, "count_prefixed only works before writes");

        self.prefixed = true;
        self.length += 4;
        self.w.write_u32::<LE>(0)?;

        Ok(self)
    }

    /// Writes raw data bytes to this node
    pub fn write_raw<R: Read>(&mut self, mut data: R) -> AnyResult<&mut Self> {
        let len = io::copy(&mut data, self.w)?;
        self.inflate(len.try_into()?)?;
        self.arbitrary = true;
        Ok(self)
    }

    /// Closure-based node builder
    /// 
    /// Example:
    /// ```
    /// use std::io;
    /// use zenit_lvl::node::{NodeName, NodeBuilder};
    /// use zenit_utils::AnyResult;
    /// 
    /// let mut out = io::Cursor::new(vec![]); // usually this is a file or smth lol
    /// let len = NodeBuilder::new(&mut out, NodeName::root())?
    ///     .build_node(NodeName::from_str("TEST"), |mut builder| {
    ///         Ok(builder
    ///             .write_raw(b"this is a test".as_ref())?
    ///             .finish()?)
    ///     })?
    ///     .finish()?;
    /// 
    /// assert_eq!(30, len);
    /// 
    /// Ok(())
    /// ```
    pub fn build_node<F>(&mut self, name: NodeName, f: F) -> AnyResult<&mut Self>
    where
        F: FnOnce(NodeBuilder<'_, W>) -> AnyResult<u32>,
    {
        let builder = NodeBuilder::new(self.w, name)?;
        let size = f(builder)?;
        self.inflate(size)?;
        self.written_nodes += 1;
        Ok(self)
    }

    pub fn write_node<R: Read>(&mut self, name: NodeName, data: R) -> AnyResult<&mut Self> {
        self.build_node(name, move |mut b| b.write_raw(data)?.finish().into())?;
        Ok(self)
    }

    pub fn align(&mut self, alignment: u32) -> AnyResult<&mut Self> {
        let cur_pos = self.w.stream_position()?;
        let new_pos = zenit_utils::align(cur_pos, alignment as u64);
        let diff = new_pos - cur_pos;
        self.write_padding(diff.try_into()?)?;
        Ok(self)
    }

    pub fn write_padding(&mut self, count: u32) -> AnyResult<&mut Self> {
        self.inflate(count)?;
        io::copy(&mut io::repeat(0).take(count as u64), self.w)?;
        Ok(self)
    }

    fn inflate(&mut self, count: u32) -> AnyResult {
        let (new_length, did_overflow) = self.length.overflowing_add(count);
        ensure!(!did_overflow, "overflow");
        self.length = new_length;
        Ok(())
    }

    /// Finishes the node writing, returning its total size (including both the payload and
    /// the header)
    pub fn finish(&mut self) -> AnyResult<u32> {
        let end = self.w.stream_position()?;

        self.w.seek(SeekFrom::Start(self.origin + 4))?;
        self.w.write_u32::<LE>(self.length)?;
        if self.prefixed {
            ensure!(!self.arbitrary, "invalid hierarchical data");
            self.w.write_u32::<LE>(self.written_nodes)?;
        }

        self.w.seek(SeekFrom::Start(end))?;

        Ok(self.length + 8)
    }
}
