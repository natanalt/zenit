use byteorder::{ReadBytesExt, LE};
use log::warn;
use std::io::Cursor;
use zenit_lvl::raw::texture::LevelTextureFormatKind;

/// Converts a specified raw texture into a compatible native format that can
/// be passed to wgpu. If possible, no conversion is done and passed data is
/// just given back.
pub fn convert_texture(
    data: Vec<u8>,
    format: LevelTextureFormatKind,
) -> Option<(Vec<u8>, wgpu::TextureFormat)> {
    let result = match format {
        LevelTextureFormatKind::DXT1 => None,
        LevelTextureFormatKind::DXT3 => None,
        LevelTextureFormatKind::A8R8G8B8 => Some((data, wgpu::TextureFormat::Rgba8Unorm)),
        LevelTextureFormatKind::R5G6B5 => {
            Some((r5g6b5_to_r8g8b8a8(&data), wgpu::TextureFormat::Rgba8Unorm))
        }
        LevelTextureFormatKind::A1R5G5B5 => {
            Some((a1r5g5b5_to_r8g8b8a8(&data), wgpu::TextureFormat::Rgba8Unorm))
        }
        LevelTextureFormatKind::A4R4G4B4 => {
            Some((a4r4g4b4_to_r8g8b8a8(&data), wgpu::TextureFormat::Rgba8Unorm))
        }
        LevelTextureFormatKind::A8 => Some((data, wgpu::TextureFormat::R8Unorm)),
        LevelTextureFormatKind::L8 => Some((data, wgpu::TextureFormat::R8Unorm)),
        LevelTextureFormatKind::A8L8 => Some((data, wgpu::TextureFormat::Rg8Unorm)),
        LevelTextureFormatKind::A4L4 => Some((a4l4_to_r8g8(&data), wgpu::TextureFormat::Rg8Unorm)),
        LevelTextureFormatKind::V8U8 => todo!(),
    };

    if result.is_none() {
        warn!("Unsupported conversion from {:?}", format);
    }

    result
}

fn r5g6b5_to_r8g8b8a8(source: &[u8]) -> Vec<u8> {
    assert!(source.len() % 2 == 0);

    let mut result = Vec::with_capacity(source.len() * 2);
    let mut input = Cursor::new(source);

    while let Ok(next) = input.read_u16::<LE>() {
        let r = ((next >> 11) & 0x1f) as u8;
        let g = ((next >> 5) & 0x3f) as u8;
        let b = ((next >> 0) & 0x1f) as u8;
        result.push(r * 8);
        result.push(g * 4);
        result.push(b * 8);
        result.push(255);
    }

    result
}

fn a1r5g5b5_to_r8g8b8a8(source: &[u8]) -> Vec<u8> {
    assert!(source.len() % 2 == 0);

    let mut result = Vec::with_capacity(source.len() * 2);
    let mut input = Cursor::new(source);

    while let Ok(next) = input.read_u16::<LE>() {
        let a = ((next >> 15) & 1) as u8;
        let r = ((next >> 10) & 0x1f) as u8;
        let g = ((next >> 5) & 0x1f) as u8;
        let b = ((next >> 0) & 0x1f) as u8;
        result.push(r * 8);
        result.push(g * 8);
        result.push(b * 8);
        result.push(a * 255);
    }

    result
}

fn a4r4g4b4_to_r8g8b8a8(source: &[u8]) -> Vec<u8> {
    assert!(source.len() % 2 == 0);

    let mut result = Vec::with_capacity(source.len() * 2);
    let mut input = Cursor::new(source);

    while let Ok(next) = input.read_u16::<LE>() {
        let a = ((next >> 12) & 0xf) as u8;
        let r = ((next >> 8) & 0xf) as u8;
        let g = ((next >> 4) & 0xf) as u8;
        let b = ((next >> 0) & 0xf) as u8;
        result.push(r * 16);
        result.push(g * 16);
        result.push(b * 16);
        result.push(a * 16);
    }

    result
}

fn a4l4_to_r8g8(source: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(source.len() * 2);
    let mut input = Cursor::new(source);

    while let Ok(next) = input.read_u8() {
        let a = (next >> 4) & 0xf;
        let l = (next >> 0) & 0xf;
        result.push(a);
        result.push(l);
    }

    result
}
