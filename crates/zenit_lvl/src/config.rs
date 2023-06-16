//! Code for accessing config node trees
//!
//! ## Introduction
//! Certain information in the BF2 engine, such as particle effects, is stored using a generic
//! config format. It's typically generated from source files structured like this:
//! ```c
//! ParticleEmitter("Something")
//! {
//!     MaxParticles(-1, -1);
//!     NoRegisterStep();
//!     SoundName("something");
//!     Spawner()
//!     {
//!         Spread()
//!         {
//!             PositionX(-0.1, 0.0);
//!             // And so on
//!         }
//!     }
//! }
//! ```
//!
//! Formally, the format can be separated into two expression types: data expressions, and scope
//! expressions.
//!
//! ### Data expressions
//! Data expressions define actual information within the config. They look like this:
//! ```c
//! // (Note: semicolons at the end are *optional*)
//! Example1();
//! Example2("string");
//! Example3(1.0, -5315.3, +inf);
//! Example4("strings and numbers", 35315, "can be mixed");
//! ```
//!
//! Every one of these expressions has a name, and an optional parameter list. The parameters have
//! two possible types: C strings, and single precision floating point numbers.
//!
//! ### Scope expressions
//! Scope expressions, denoted by curly brackets `{}`, are just containers for other data and scope
//! expressions.
//!
//! In BF2 structures, scope expressions are typically prefixed by a data expression defining their
//! contents, like this:
//! ```c
//! ParticleEmitter("Something")
//! {
//!     Data()
//!     Data()
//!     Data()
//! }
//! ```
//! That being said, it's a convention, and a config file like this is also valid:
//! ```c
//! // In other words, putting something before a scope isn't necessary.
//! // Scopes function as standalone objects.
//! {
//!     NamelessScope("how scary")
//! }
//! {
//!     AnotherScope("holy hell")
//! }
//! ```
//!
//! ## Internal representation
//! A config node fits into the hierarchical chunk structure of level files. The 3 node names
//! you will encounter in a config node are `NAME`, `DATA` and `SCOP`.
//!
//! * `NAME` contains the hashed name of the object. It's usually determined from the source filename.
//! * `DATA` contains all information stored in a data statement inside its payload. Its format is
//!    described below.
//! * `SCOP` contains additional nested `DATA` and `SCOP` nodes, it directly corresponds to a scope
//!    expression.
//!
//! ### `DATA` format
//! Here's a pseudocode definition of the structure:
//! ```c
//! struct DataExpression {
//!     u32 name_hash;
//!
//!     u8  value_count;
//!     u32 values[value_count];
//!     
//!     u32 tail_length;
//!     u8  tail[tail_length];
//! }
//! ```
//!
//! Any parameters are defined in the data expression are represented one by one in the `values`
//! list. What each `values` entry contains is dependent on the parameter's type (note, that the
//! parameter types themselves are *not* encoded, so parsing code needs to be aware of what types
//! to expect).
//!
//! So, how are the two value types stored?
//!
//! #### Floating point storage
//! This one's simple. You just reinterpret the 32-bit value as an `f32` float.
//!
//! #### String storage
//! This one gets more complicated. It *seems* that the value is meant to be interpreted as an
//! offset in the chunk, *itself offset by 9*. To make it clearer, a value of `23` will imply an
//! offset of `23+9=32` within the `DATA` chunk.
//!
//! Does that make sense? Hopefully :3
//!

use crate::node::{read_node_children, NodeHeader, NodeRead, NodeWrite, NodeWriter};
use anyhow::{bail, ensure};
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use std::{
    ffi::{CStr, CString},
    io::{Read, Seek, Write},
};
use zenit_utils::{ok, packed::PackedData, AnyResult, AsciiDisplay};

/// Represents a config munged node.
///
/// See module documentation for explanation.
#[derive(Debug, Clone)]
pub struct LevelConfig {
    /// Hashed name of the config node
    pub name_hash: u32,
    /// Fake scope containing all config expressions from the node.
    pub root: ConfigScope,
}

impl NodeRead for LevelConfig {
    fn read_node_payload<R: Read + Seek>(r: &mut R, meta: NodeHeader) -> AnyResult<Self> {
        let mut name_hash = None;
        let raw_children = read_node_children(r, meta)?;
        let mut children = Vec::with_capacity(raw_children.len() - 1);

        for child in raw_children {
            if child.name == b"NAME" {
                ensure!(name_hash.is_none(), "duplicate name");
                ensure!(child.size == 4, "invalid config node name");
                child.seek_to_payload(r)?;
                name_hash = Some(r.read_u32::<LE>()?);
            } else if child.name == b"DATA" {
                children.push(ConfigExpr::Data(ConfigData::read_node_at(r, meta)?));
            } else if child.name == b"SCOP" {
                children.push(ConfigExpr::Scope(ConfigScope::read_node_at(r, meta)?));
            } else {
                bail!(
                    "unexpected config node: `{}`",
                    AsciiDisplay(child.name.as_ref())
                );
            }
        }

        Ok(Self {
            name_hash: match name_hash {
                Some(value) => value,
                None => bail!("missing name"),
            },
            root: ConfigScope { children },
        })
    }
}

impl NodeWrite for LevelConfig {
    fn write_node<W: Write + Seek>(&self, writer: &mut NodeWriter<W>) -> AnyResult {
        writer.write_node(b"NAME", self.name_hash)?;
        self.root.write_node(writer)?;
        ok()
    }
}

/// Represents a config expression, either data or a scope.
#[derive(Debug, Clone)]
pub enum ConfigExpr {
    Data(ConfigData),
    Scope(ConfigScope),
}

impl NodeRead for ConfigExpr {
    fn read_node_payload<R: Read + Seek>(r: &mut R, meta: NodeHeader) -> AnyResult<Self> {
        if meta.name == b"DATA" {
            Ok(Self::Data(ConfigData::read_node_payload(r, meta)?))
        } else if meta.name == b"SCOP" {
            Ok(Self::Scope(ConfigScope::read_node_payload(r, meta)?))
        } else {
            bail!(
                "unexpected config scope node: `{}`",
                AsciiDisplay(meta.name.as_ref())
            )
        }
    }
}

impl NodeWrite for ConfigExpr {
    fn write_node<W: Write + Seek>(&self, writer: &mut NodeWriter<W>) -> AnyResult {
        // TODO: get rid of the clone once PackedNode is spread into separate Read/Write traits
        match self.clone() {
            ConfigExpr::Data(data) => writer.write_node(b"DATA", data),
            ConfigExpr::Scope(scope) => writer.write_node(b"SCOP", scope),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ConfigScope {
    pub children: Vec<ConfigExpr>,
}

impl NodeRead for ConfigScope {
    fn read_node_payload<R: Read + Seek>(r: &mut R, meta: NodeHeader) -> AnyResult<Self> {
        Ok(Self {
            children: read_node_children(r, meta)?
                .into_iter()
                .map(|child| ConfigExpr::read_node_at(r, child))
                .collect::<AnyResult<_>>()?,
        })
    }
}

impl NodeWrite for ConfigScope {
    fn write_node<W: Write + Seek>(&self, writer: &mut NodeWriter<W>) -> AnyResult {
        for child in &self.children {
            child.write_node(writer)?;
        }
        ok()
    }
}

#[derive(Debug, Clone)]
pub struct ConfigData {
    /// Hashed name of the data expression
    pub name_hash: u32,
    /// Tuple-like values stored in this data object. Use [`ConfigData::get`] for actual reading.
    pub values: Vec<u32>,
    /// Data after the value list
    pub tail: Vec<u8>,
}

impl ConfigData {
    /// Attempts to read a value from the object. The generic type parameter has implementations
    /// for [`f32`] and [`CString`] which is all data chunks can store anyway.
    pub fn get<T: ConfigValue>(&self, idx: u32) -> Option<T> {
        T::get(self, idx)
    }

    /// Calculates the amount of bytes in the header before the tail.
    pub fn size_before_tail(&self) -> usize {
        // sizeof(name_hash) + sizeof(value_count) + sizeof(tail_length)
        let constant_size = 4 + 1 + 4;
        let values_size = self.values.len() * 4;
        values_size + constant_size
    }
}

// Implementing PackedData instead of NodeData, since
// NodeData is implemented for all PackedData
impl PackedData for ConfigData {
    fn read_packed<R: Read>(r: &mut R) -> AnyResult<Self> {
        let name_hash = r.read_u32::<LE>()?;

        let value_count = r.read_u8()? as usize;
        let mut values = Vec::with_capacity(value_count);
        for _ in 0..value_count {
            values.push(r.read_u32::<LE>()?);
        }

        let tail_length = r.read_u32::<LE>()? as usize;
        let mut tail = vec![0; tail_length];
        r.read_exact(&mut tail)?;

        Ok(Self {
            name_hash,
            values,
            tail,
        })
    }

    fn write_packed<W: Write>(&self, w: &mut W) -> AnyResult {
        w.write_u32::<LE>(self.name_hash)?;

        // Alright, so I wouldn't be surprised if BF2 put a theoretical parameter
        // limit at a *signed* byte size, rather than an unsigned one.
        //
        // Although does it really matter if the limit is 127 or 255? I mean, who
        // would even dare to put so many parameters in a config expression...
        //
        // Just to be sure, I'm going to make the limit 126.
        ensure!(self.values.len() <= 126, "too many parameter values");

        w.write_u8(self.values.len() as u8)?;
        for &value in &self.values {
            w.write_u32::<LE>(value)?;
        }

        // Same as above with value limit
        ensure!(self.tail.len() < i32::MAX as usize, "tail too big");
        w.write_u32::<LE>(self.tail.len() as u32)?;
        w.write_all(&self.tail)?;

        ok()
    }
}

/// Magic implementation stuff for [`ConfigData::get`]. Don't worry about it, you likely won't need
/// to ever implement it yourself.
pub trait ConfigValue: Sized {
    fn get(data: &ConfigData, idx: u32) -> Option<Self>;
}

impl ConfigValue for CString {
    fn get(data: &ConfigData, index: u32) -> Option<Self> {
        let index = index as usize;

        // Calculate the string's absolute offset in the data chunk
        // (See format docs for details)
        let raw_offset = data.values.get(index).cloned()? as usize;
        let absolute_offset = raw_offset + 9;

        // We make the amazing assumption that the offset actually points to
        // the tail, since that's the only raw block we store in the structure.
        let header_size = data.size_before_tail();
        if absolute_offset < header_size {
            return None;
        }
        let in_tail_offset = absolute_offset - header_size;

        // Convert it to a string, and verify the presence of a nul byte at the end.
        // (Hopefully no code depends on a different string layout)
        Some(
            CStr::from_bytes_with_nul(&data.tail[in_tail_offset..])
                .ok()?
                .into(),
        )
    }
}

impl ConfigValue for f32 {
    fn get(data: &ConfigData, index: u32) -> Option<Self> {
        data.values.get(index as usize).map(|&raw| {
            // Reinterpret the value as a float
            f32::from_bits(raw)
        })
    }
}
