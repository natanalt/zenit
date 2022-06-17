use byteorder::{ReadBytesExt, LE};
use glam::UVec2;
use log::warn;
use std::io::Cursor;
use zenit_lvl::raw::texture::LevelTextureFormatKind;
use zenit_utils::color::{RGB8, RGBA8};

pub fn convert_format(format: LevelTextureFormatKind) -> Option<wgpu::TextureFormat> {
    match format {
        LevelTextureFormatKind::DXT1 => None,
        LevelTextureFormatKind::DXT3 => None,
        LevelTextureFormatKind::A8R8G8B8 => Some(wgpu::TextureFormat::Rgba8Unorm),
        LevelTextureFormatKind::R5G6B5 => Some(wgpu::TextureFormat::Rgba8Unorm),
        LevelTextureFormatKind::A1R5G5B5 => Some(wgpu::TextureFormat::Rgba8Unorm),
        LevelTextureFormatKind::A4R4G4B4 => Some(wgpu::TextureFormat::Rgba8Unorm),
        LevelTextureFormatKind::A8 => None,
        LevelTextureFormatKind::L8 => None,
        LevelTextureFormatKind::A8L8 => None,
        LevelTextureFormatKind::A4L4 => None,
        LevelTextureFormatKind::V8U8 => None,
    }
}

/// Converts a specified raw texture into a compatible native format that can
/// be passed to wgpu. Use [`convert_format`] to get what format the texture
/// has been converted to.
pub fn convert_texture(
    data: Vec<u8>,
    size: UVec2,
    format: LevelTextureFormatKind,
) -> Option<Vec<u8>> {
    use LevelTextureFormatKind as FormatKind;

    if format == FormatKind::DXT1 || format == FormatKind::DXT3 {
        warn!("DXT/S3 compression is not supported (yet?)");
        return None;
    }

    // TODO: support all other texture formats (and figure out their exact storage format)
    match format {
        FormatKind::A8R8G8B8 => Some(into_rgba8(size, A8R8G8B8Decoder::new(&data))),
        FormatKind::R5G6B5 => Some(into_rgba8(
            size,
            R5G6B5Decoder::new(&data).map(Into::<RGBA8>::into),
        )),
        FormatKind::A1R5G5B5 => Some(into_rgba8(size, A1R5G5B5Decoder::new(&data))),
        FormatKind::A4R4G4B4 => Some(into_rgba8(size, A4R4G4B4Decoder::new(&data))),
        kind => {
            warn!("Texture type `{:?}` is not yet supported!", kind);
            None
        }
    }
}

macro_rules! decoder_template {
    ($name:ident, $doc:literal, $($next:tt)*) => {
        #[doc = $doc]
        pub struct $name <'a>(Cursor<&'a [u8]>);
        impl<'a> $name<'a> {
            pub fn new(source: &'a [u8]) -> Self {
                Self(Cursor::new(source))
            }
        }
        impl<'a> Iterator for $name<'a> $($next)*
    };
}

// Needed because no wgpu texture format accepts this sort of data ordering
decoder_template!(
    A8R8G8B8Decoder,
    "Decodes a Direct3D A8R8G8B8 little endian texture into RGBA8",
    {
        type Item = RGBA8;
        fn next(&mut self) -> Option<Self::Item> {
            let a = self.0.read_u8().ok()?;
            let r = self.0.read_u8().ok()?;
            let g = self.0.read_u8().ok()?;
            let b = self.0.read_u8().ok()?;
            Some(RGBA8 { r, g, b, a })
        }
    }
);

decoder_template!(
    R5G6B5Decoder,
    "Decodes a Direct3D R5G6B5 little endian texture into RGB8",
    {
        type Item = RGB8;
        fn next(&mut self) -> Option<Self::Item> {
            let next = self.0.read_u16::<LE>().ok()?;
            let r = ((next >> 11) & 0x1f) as u8 * 8;
            let g = ((next >> 5) & 0x3f) as u8 * 4;
            let b = ((next >> 0) & 0x1f) as u8 * 8;
            Some(RGB8 { r, g, b })
        }
    }
);

decoder_template!(
    A1R5G5B5Decoder,
    "Decodes a Direct3D A1R5G5B5 little endian texture into RGBA8",
    {
        type Item = RGBA8;
        fn next(&mut self) -> Option<Self::Item> {
            let next = self.0.read_u16::<LE>().ok()?;
            let a = ((next >> 15) & 1) as u8 * 255;
            let r = ((next >> 10) & 0x1f) as u8 * 8;
            let g = ((next >> 5) & 0x1f) as u8 * 8;
            let b = ((next >> 0) & 0x1f) as u8 * 8;
            Some(RGBA8 { r, g, b, a })
        }
    }
);

decoder_template!(
    A4R4G4B4Decoder,
    "Decodes a Direct3D A4R4G4B4 little endian texture into RGBA8",
    {
        type Item = RGBA8;
        fn next(&mut self) -> Option<Self::Item> {
            let next = self.0.read_u16::<LE>().ok()?;
            let a = ((next >> 12) & 0xf) as u8;
            let r = ((next >> 8) & 0xf) as u8;
            let g = ((next >> 4) & 0xf) as u8;
            let b = ((next >> 0) & 0xf) as u8;
            Some(RGBA8 { r, g, b, a })
        }
    }
);

#[inline]
pub fn into_rgba8(res: UVec2, data: impl Iterator<Item = RGBA8>) -> Vec<u8> {
    let mut result = Vec::with_capacity((res.x * res.y * 4) as usize);
    for pixel in data {
        result.push(pixel.r);
        result.push(pixel.g);
        result.push(pixel.b);
        result.push(pixel.a);
    }
    result
}

#[inline]
pub fn into_rgb8(res: UVec2, data: impl Iterator<Item = RGB8>) -> Vec<u8> {
    let mut result = Vec::with_capacity((res.x * res.y * 3) as usize);
    for pixel in data {
        result.push(pixel.r);
        result.push(pixel.g);
        result.push(pixel.b);
    }
    result
}
