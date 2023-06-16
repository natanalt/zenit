//! Utilities for dealing with BF2's level nodes (also known as chunks)
//!
//! Provides raw reading/writing capabilities, and a derive macro for creating automatic parsers
//! of node hierarchical structures.

use anyhow::{bail, ensure};
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use std::io::{self, Read, Seek, SeekFrom, Write};
use zenit_utils::{
    ok,
    packed::{PackedData, PackedWriteExt},
    AnyResult, SeekableTakeExt,
};

pub use zenit_lvl_proc::NodeData;

/// Represents a 4-byte name of the chunk. The chunk can either be a short ASCII string, or
/// an arbitrary 32-bit number (like a hash). In the latter case, the number is little endian
/// encoded.
#[doc(alias = "chunk")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeName(pub [u8; 4]);

impl NodeName {
    /// Converts given string into a [`NodeName`].
    ///
    /// ## Panics
    /// Panics if the string isn't 4 bytes long.
    pub const fn from_str(s: &str) -> Self {
        let bytes = s.as_bytes();
        assert!(bytes.len() == 4, "invalid string length");
        Self([bytes[0], bytes[1], bytes[2], bytes[3]])
    }

    pub const fn as_bytes(&self) -> &[u8; 4] {
        &self.0
    }
}

impl<'a> From<&'a [u8; 4]> for NodeName {
    fn from(value: &'a [u8; 4]) -> Self {
        Self(value.clone())
    }
}

impl From<u32> for NodeName {
    fn from(value: u32) -> Self {
        Self(value.to_le_bytes())
    }
}

impl Into<u32> for NodeName {
    fn into(self) -> u32 {
        u32::from_le_bytes(self.0)
    }
}

impl AsRef<[u8]> for NodeName {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl<'a> PartialEq<&'a [u8]> for NodeName {
    fn eq(&self, other: &&'a [u8]) -> bool {
        &self.0 == other
    }
}

impl<'a> PartialEq<&'a [u8; 4]> for NodeName {
    fn eq(&self, other: &&'a [u8; 4]) -> bool {
        &self.0 == &other[0..4]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeHeader {
    pub header_position: u64,
    pub name: NodeName,
    pub size: u32,
}

impl NodeHeader {
    pub fn seek_to_payload(&self, r: &mut impl Seek) -> io::Result<()> {
        r.seek(io::SeekFrom::Start(self.header_position + 8))?;
        ok()
    }
}

/// Trait for node reading/writing node hierarchies. Can be derived.
pub trait NodeData: NodeRead + NodeWrite {}
impl<T: NodeRead + NodeWrite> NodeData for T {}

pub trait NodeRead: Sized + Clone {
    /// Processes the node's payload and returns this type's instance.
    ///
    /// The reader is placed at the first byte of the payload by the caller, that is right
    /// after the header. Its final seek position is unspecified.
    ///
    /// Direct usage of this function is discouraged as it is incredibly error prone.
    fn read_node_payload<R: Read + Seek>(r: &mut R, meta: NodeHeader) -> AnyResult<Self>;

    /// Wrapper around [`Self::read_node_payload`], that pre-seeks into the payload. Otherwise,
    /// the behavior matches that function.
    fn read_node_at<R: Read + Seek>(r: &mut R, meta: NodeHeader) -> AnyResult<Self> {
        meta.seek_to_payload(r)?;
        Ok(Self::read_node_payload(
            &mut r.seekable_take(meta.size as u64),
            meta,
        )?)
    }
}

pub trait NodeWrite: Sized + Clone {
    /// Writes the node into the given writer. The caller must not call `NodeWriter::finish`
    /// by themselves.
    fn write_node<W: Write + Seek>(&self, w: &mut NodeWriter<W>) -> AnyResult;
}

/// Blanket implementation of [`NodeData`] for every [`PackedData`]. It works about as can be
/// expected, treating the payload of a node as raw binary data.
impl<T: PackedData> NodeRead for T {
    fn read_node_payload<R: Read + Seek>(r: &mut R, _: NodeHeader) -> AnyResult<Self> {
        Ok(T::read_packed(r)?)
    }
}

impl<T: PackedData> NodeWrite for T {
    fn write_node<W: Write + Seek>(&self, writer: &mut NodeWriter<W>) -> AnyResult {
        writer.write_packed(self.clone())?;
        ok()
    }
}

/// A very specific [`NodeData`] implementation that doesn't actually read anything inside of itself.
/// What it instead does, is cache the node's header information that can be lazily read later.
///
/// The main use case is allowing for conditional loading of large game resources, such as textures.
#[derive(Debug, Clone)]
pub enum LazyData<T: NodeData> {
    /// Represents a [`LazyData`] instance after loading, with cached metadata.
    Read(NodeHeader),
    /// Represents a [`LazyData`] instance ready for writing. This is only way I could get the write
    /// interface to work with this, even if a bit hacky.
    Write(T),
}

impl<T: NodeData> LazyData<T> {
    pub fn read_cached<R: Read + Seek>(&self, r: &mut R) -> AnyResult<T> {
        match self {
            LazyData::Read(header) => Ok(T::read_node_at(r, *header)?),
            LazyData::Write(_) => bail!("node not cached for reading"),
        }
    }
}

impl<T: NodeData> NodeRead for LazyData<T> {
    fn read_node_payload<R: Read + Seek>(_: &mut R, meta: NodeHeader) -> AnyResult<Self> {
        Ok(Self::Read(meta))
    }
}

impl<T: NodeData> NodeWrite for LazyData<T> {
    fn write_node<W: Write + Seek>(&self, writer: &mut NodeWriter<W>) -> AnyResult {
        match self {
            LazyData::Read(_) => panic!("cannot write a LazyData::Read"),
            LazyData::Write(data) => Ok(data.write_node(writer)?),
        }
    }
}

impl<T: NodeData> From<NodeHeader> for LazyData<T> {
    fn from(value: NodeHeader) -> Self {
        LazyData::Read(value)
    }
}

impl<T: NodeData> From<T> for LazyData<T> {
    fn from(value: T) -> Self {
        LazyData::Write(value)
    }
}

pub fn read_node_header<R>(r: &mut R) -> io::Result<NodeHeader>
where
    R: Read + Seek,
{
    Ok(NodeHeader {
        header_position: r.stream_position()?,
        name: {
            let mut name = NodeName([0; 4]);
            r.read_exact(&mut name.0)?;
            name
        },
        size: r.read_u32::<LE>()?,
    })
}

// TODO: move read_node_* functions to NodeHeader

/// Reads the given node's payload as a byte buffer.
pub fn read_node_payload<R>(r: &mut R, header: NodeHeader) -> io::Result<Vec<u8>>
where
    R: Read + Seek,
{
    let mut result = Vec::with_capacity(header.size as usize);
    r.seek(SeekFrom::Start(header.header_position + 8))?;
    r.take(header.size as u64).read_to_end(&mut result)?;
    Ok(result)
}

/// Attempts to parse the given node as a parent node with a list of child nodes.
pub fn read_node_children<R>(r: &mut R, header: NodeHeader) -> io::Result<Vec<NodeHeader>>
where
    R: Read + Seek,
{
    // Attempt #1: assume there is no node count prefix
    if let Ok(children) = read_children_impl(r, header, false) {
        return Ok(children);
    }

    // Attempt #2: assume there is a node count prefix
    if let Ok(children) = read_children_impl(r, header, true) {
        return Ok(children);
    }

    // Nothing more we can do
    Err(io::Error::new(
        io::ErrorKind::InvalidData,
        "node doesn't seem to have a hierarchy",
    ))
}

fn read_children_impl<R>(
    r: &mut R,
    header: NodeHeader,
    count_prefix: bool,
) -> io::Result<Vec<NodeHeader>>
where
    R: Read + Seek,
{
    r.seek(SeekFrom::Start(header.header_position + 8))?;
    let mut r = r.take(header.size as u64); // TODO: seekable take here

    let mut remaining_nodes = match count_prefix {
        true => Some(r.read_u32::<LE>()?),
        false => None,
    };

    let mut children = match remaining_nodes {
        Some(nodes) => Vec::with_capacity(nodes as usize),
        None => Vec::new(),
    };

    'top: loop {
        // Skip any padding of 0s
        // A bit ugly solution but eh
        'padding: loop {
            match r.read_u8() {
                // Skip padding byte
                Ok(0) => {}

                // Found a non-zero byte, rewind and break out of the padding loop
                Ok(_) => {
                    // Increase the limit back
                    r.set_limit(r.limit() + 1);
                    r.get_mut().seek(SeekFrom::Current(-1))?;
                    break 'padding;
                }

                // Out of data, break out of the top loop
                Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break 'top,

                // Other IO error, return it
                Err(e) => return Err(e),
            }
        }

        // Verify the remaining node count
        if remaining_nodes == Some(0) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unknown data after exhausted node count",
            ));
        }

        // Read the node header
        let header_position = r.get_mut().stream_position()?;
        let name = r.read_u32::<LE>()?;
        let size = r.read_u32::<LE>()?;

        if r.limit() < size as u64 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "node went out of bounds",
            ));
        }

        r.set_limit(r.limit() - size as u64);
        r.get_mut().seek(SeekFrom::Current(size as i64)).unwrap();

        if let Some(remaining_nodes) = remaining_nodes.as_mut() {
            *remaining_nodes -= 1;
        }

        children.push(NodeHeader {
            header_position,
            name: NodeName(name.to_le_bytes()),
            size,
        })
    }

    Ok(children)
}

/// A builder-style node writer.
///
/// ## Quick crash course
///  1. You can use [`NodeWriter::build_node`] to create a nested node writer for a child node.
///  2. You can use [`NodeWriter::write_node`] to create a child node and fill it with a [`PackedData`]
///  3. You can write raw data into the node using its [`Write`] implementation, that just forwards
///     everything to the parent writer.
///     object.
///  4. Once you're finished, either call [`NodeWriter::finish`] manually or drop the writer. Note,
///     that the drop implementations unwraps any IO errors returned by `finish`.
pub struct NodeWriter<'w, W: Write + Seek> {
    w: &'w mut W,
    data_start: u64,
    finished: bool,
}

impl<'w, W: Write + Seek> NodeWriter<'w, W> {
    pub fn new(w: &'w mut W, name: impl Into<NodeName>) -> AnyResult<Self> {
        let data_start = w.stream_position()?;
        w.write_all(&name.into().0)?;
        w.write_u32::<LE>(0)?;

        Ok(Self {
            w,
            data_start,
            finished: false,
        })
    }

    /// Creates a new node, with contents from a [`PackedData`] or [`NodeData`] object.
    pub fn write_node(&mut self, name: impl Into<NodeName>, data: impl NodeData + 'w) -> AnyResult {
        assert!(!self.finished);
        self.build_node(name.into(), move |writer| {
            data.write_node(writer)?;
            ok()
        })?;
        ok()
    }

    /// Creates a nested node builder.
    pub fn build_node<'a, N, F>(&'a mut self, name: N, f: F) -> AnyResult
    where
        N: Into<NodeName>,
        F: FnOnce(&mut NodeWriter<'a, W>) -> AnyResult,
        'w: 'a,
    {
        assert!(!self.finished);

        let mut writer = NodeWriter::new(self.w, name.into())?;
        (f)(&mut writer)?;
        writer.finish()?;

        ok()
    }

    /// Finishes writing the node, by marking its final size in the stream.
    pub fn finish(&mut self) -> AnyResult {
        if self.finished {
            return ok();
        }

        let data_end = self.w.stream_position()?;
        let data_size = data_end - self.data_start - 8;
        ensure!(data_size <= u32::MAX.into(), "node too large");

        self.w.seek(SeekFrom::Start(self.data_start + 4))?;
        self.w.write_u32::<LE>(data_size as u32)?;
        self.w.seek(SeekFrom::Start(data_end))?;

        self.finished = true;
        ok()
    }

    /// Borrows the underlying writer.
    ///
    /// ## How to not break things
    /// You are allowed to write and seek using this writer, but there's a catch:
    /// seeking is a dangerous ordeal - you must not seek before the node's start,
    /// or the code will break, most likely panic.
    ///
    /// You can use the seeking to rewind, or skip parts of the writer, and `finish`
    /// will account for that as unused/gained space in its final size calculation.
    pub fn borrow_writer(&mut self) -> &mut W {
        &mut self.w
    }
}

impl<'w, W: Write + Seek> Write for NodeWriter<'w, W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.w.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.w.flush()
    }

    fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
        self.w.write_vectored(bufs)
    }

    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.w.write_all(buf)
    }

    fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> std::io::Result<()> {
        self.w.write_fmt(fmt)
    }
}

impl<'w, W: Write + Seek> Drop for NodeWriter<'w, W> {
    fn drop(&mut self) {
        if !self.finished {
            self.finish().unwrap();
        }
    }
}
