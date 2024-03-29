use crate::node::{LazyData, NodeData};
use bitflags::bitflags;
use byteorder::{ReadBytesExt, WriteBytesExt, LE};
use std::{
    ffi::CString,
    io::{Read, Write},
};
use zenit_proc::{ext_repr, PackedData};
use zenit_utils::{ok, packed::PackedData, AnyResult};

#[derive(Debug, Clone, NodeData)]
pub struct LevelModel {
    #[node("NAME")]
    pub name: CString,
    #[node("VRTX")]
    pub vertex: u32, // ?
    #[node("NODE")]
    pub node: CString,
    #[node("INFO")]
    pub info: ModelInfo,
    #[nodes("segm")]
    pub segments: Vec<ModelSegment>,
    #[node("SPHR")]
    pub sphere: LevelModelSphere,
}

/// Proceed with caution, the layout may not be what it appears to be.
/// Or maybe it's all correct, I have no clue
#[derive(Debug, Clone, PackedData)]
pub struct ModelInfo {
    pub unknown0x00: u32,
    pub unknown0x04: u32,
    pub unknown0x08: u32,
    pub unknown0x0c: u32,
    pub vertex_box: [[f32; 3]; 2],
    pub visibility_box: [[f32; 3]; 2],
    pub unknown0x40: u32, // ?
    pub face_count: u32,  // ?
}

#[derive(Debug, Clone, NodeData)]
pub struct ModelSegment {
    #[node("INFO")]
    pub info: ModelInfo,
    #[node("MTRL")]
    pub material: ModelMaterial,
    #[node("RTYP")]
    pub render_type: CString,
    #[nodes("TNAM")]
    pub texture_names: [ModelTextureName; 4],
    #[node("BBOX")]
    pub aabb: ModelSegmentAABB,
    #[node("IBUF")]
    pub index_buffer: LazyData<Vec<u8>>,
    #[nodes("VBUF")]
    pub vertex_buffers: Vec<LazyData<Vec<u8>>>,
    #[node("BNAM")]
    pub bone_map_name: CString,
}

#[derive(Debug, Clone, PackedData)]
pub struct ModelSegmentInfo {
    pub topology: ModelSegmentTopology,
    pub vertex_count: u32,
    pub primitive_count: u32,
}

#[derive(Debug, Clone, PartialEq, PackedData)]
#[ext_repr(u32)]
#[parse_as(u32)]
pub enum ModelSegmentTopology {
    PointList = 1,
    LineList = 2,
    LineStrip = 3,
    TriangleList = 4,
    TriangleStrip = 5,
    TriangleFan = 6,
}

#[derive(Debug, Clone, PackedData)]
pub struct ModelMaterial {
    pub flags: MaterialFlags,
    pub diffuse_color: [u8; 4],
    pub specular_color: [u8; 4],
    pub specular_exponent: u32,
    pub parameters: [u32; 2],
    pub attached_light: CString,
}

bitflags! {
    pub struct MaterialFlags: u32 {
        const NORMAL = 1 << 0;
        const HARD_EDGED = 1 << 1;
        const TRANSPARENT = 1 << 2;
        const GLOSS_MAP = 1 << 3;
        const GLOW = 1 << 4;
        const NORMAL_MAP = 1 << 5;
        const ADDITIVE = 1 << 6;
        const SPECULAR = 1 << 7;
        const ENVIRONMENT_MAP = 1 << 8;
        const VERTEX_LIGHTING = 1 << 9;
        const TILED_NORMAL_MAP = 1 << 11;
        const DOUBLE_SIDED = 1 << 16;
        const SCROLLING = 1 << 24;
        const ENERGY = 1 << 25;
        const ANIMATED = 1 << 26;
        const ATTACHED_LIGHT = 1 << 27;
    }
}

// TODO: derive PackedDa for bitflags generated types
impl PackedData for MaterialFlags {
    fn read_packed<R: Read>(r: &mut R) -> AnyResult<Self> {
        Ok(r.read_u32::<LE>()?.into())
    }

    fn write_packed<W: Write>(&self, w: &mut W) -> AnyResult {
        w.write_u32::<LE>(self.bits())?;
        ok()
    }
}

impl From<u32> for MaterialFlags {
    fn from(value: u32) -> Self {
        Self::from_bits_truncate(value)
    }
}

#[derive(Debug, Clone, PackedData)]
pub struct ModelTextureName {
    pub index: u32,
    pub name: CString,
}

#[derive(Debug, Clone, PackedData)]
pub struct ModelSegmentAABB {
    pub min: [f32; 3],
    pub max: [f32; 3],
}

/// No idea what this does, I assume the layout is supposed to mean that
#[derive(Debug, Clone, PackedData)]
pub struct LevelModelSphere {
    pub position: [f32; 3],
    pub radius: f32,
}
