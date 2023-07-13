use crate::{
    graphics::{
        CubemapDescriptor, CubemapFace, CubemapHandle, CubemapWriteDescriptor, TextureDescriptor,
        TextureHandle, TextureWriteDescriptor,
    },
    scene::EngineBorrow,
};
use glam::uvec2;
use itertools::Itertools;
use std::io::{Read, Seek};
use thiserror::Error;
use wgpu::TextureFormat;
use zenit_lvl::{
    game::{D3DFormat, LevelTexture, LevelTextureFormat, LevelTextureFormatInfo, LevelTextureKind},
    node::{NodeHeader, NodeRead},
};
use log::*;

pub enum LoadedTexture {
    Texture(TextureHandle),
    Cubemap(CubemapHandle),
}

#[derive(Debug, Error)]
pub enum TextureLoadError {
    #[error("the texture had an invalid name")]
    BadName,
    #[error("no suitable format found in the texture")]
    NoFormat,
    #[error("the texture node had an improper amount of defined faces")]
    BadFaceAmount,
    #[error("a node parsing error occurred: {0:#?}")]
    ParseError(anyhow::Error),
    #[error("a texture read error occurred: {0:#?}")]
    ReadError(anyhow::Error),
}

/// Loads a texture and registers it inside the asset manager.
/// 
/// Any errors are logged, but not returned back. For better control, you may want to use
/// [`load_texture`] instead.
pub fn load_texture_as_asset(
    (mut r, node): (impl Read + Seek, NodeHeader),
    engine: &mut EngineBorrow,
) {
    let (name, texture) = match load_texture((&mut r, node), engine) {
        Ok(v) => v,
        Err(e) => {
            error!("An error occurred while loading a texture: {e:#?}");
            return;
        }
    };

    match texture {
        LoadedTexture::Texture(texture) => {
            trace!("Loaded texture `{name}`...");
            engine.assets.textures.insert(name, texture);
        }
        LoadedTexture::Cubemap(cubemap) => {
            trace!("Loaded cubemap `{name}`...");
            engine.assets.cubemaps.insert(name, cubemap);
        }
    }
}

/// Loads a texture without registering it inside the asset manager.
pub fn load_texture(
    (mut r, node): (impl Read + Seek, NodeHeader),
    engine: &mut EngineBorrow,
) -> Result<(String, LoadedTexture), TextureLoadError> {
    use TextureLoadError::*;

    let level_texture = LevelTexture::read_node_at(&mut r, node).map_err(|err| ParseError(err))?;
    let texture_name = level_texture.name.into_string().map_err(|_| BadName)?;
    let level_format = level_texture
        .formats
        .into_iter()
        .map(|format| (rank_texture_format(&format, engine), format))
        .filter(|(rank, _)| *rank >= 0) // Disqualify any format ranked below zero
        .max_by_key(|(rank, _)| *rank)
        .map(|(_, format)| format)
        .ok_or(NoFormat)?;

    let LevelTextureFormat { info, faces, unfiltered } = level_format;

    let renderer = &mut engine.renderer;
    let format = d3dformat_to_wgpu(info.format);

    Ok(match info.kind {
        LevelTextureKind::D2 => {
            let texture = renderer.create_texture(&TextureDescriptor {
                name: texture_name.clone(),
                size: uvec2(info.width as u32, info.height as u32),
                mip_levels: info.mipmaps as u32,
                format,
                unfiltered: unfiltered.len() > 0,
            });

            if faces.len() != 1 {
                return Err(BadFaceAmount);
            }

            for mipmap in &faces[0].mipmaps {
                engine.renderer.write_texture(&TextureWriteDescriptor {
                    handle: &texture,
                    mip_level: mipmap.info.mip_level,
                    data: &convert_texture_format(
                        &info,
                        mipmap
                            .body
                            .read_cached(&mut r)
                            .map_err(|err| ReadError(err))?,
                    ),
                })
            }

            (texture_name, LoadedTexture::Texture(texture))
        }
        LevelTextureKind::Cubemap => {
            let cubemap = renderer.create_cubemap(&CubemapDescriptor {
                name: texture_name.clone(),
                size: uvec2(info.width as u32, info.height as u32),
                mip_levels: info.mipmaps as u32,
                format,
            });

            if faces.len() != 6 {
                return Err(BadFaceAmount);
            }

            for (index, face_info) in faces.into_iter().enumerate() {
                let face = CubemapFace::try_from(index as u8).unwrap();

                for mipmap in face_info.mipmaps {
                    engine.renderer.write_cubemap(&CubemapWriteDescriptor {
                        handle: &cubemap,
                        face,
                        mip_level: mipmap.info.mip_level,
                        data: &convert_texture_format(
                            &info,
                            mipmap
                                .body
                                .read_cached(&mut r)
                                .map_err(|err| ReadError(err))?,
                        ),
                    })
                }
            }

            (texture_name, LoadedTexture::Cubemap(cubemap))
        }
    })
}

fn rank_texture_format(format: &LevelTextureFormat, engine: &mut EngineBorrow) -> i32 {
    let compression_score = if engine.renderer.capabilities.allow_bc_compression {
        10
    } else {
        i32::MIN
    };

    match format.info.format {
        D3DFormat::DXT1 => compression_score,
        D3DFormat::DXT3 => compression_score,
        D3DFormat::A8R8G8B8 => 5,
        D3DFormat::R8G8B8 => 4,
        D3DFormat::R5G6B5 => 3,
        D3DFormat::A1R5G5B5 => 1,
        D3DFormat::A4R4G4B4 => 1,
        D3DFormat::A8 => 1,
        D3DFormat::L8 => 1,
        D3DFormat::A8L8 => 1,
        D3DFormat::A4L4 => 1,
        D3DFormat::V8U8 => 1,
    }
}

fn d3dformat_to_wgpu(d3d: D3DFormat) -> TextureFormat {
    match d3d {
        D3DFormat::DXT1 => TextureFormat::Bc1RgbaUnorm,
        D3DFormat::DXT3 => TextureFormat::Bc2RgbaUnorm,
        D3DFormat::A8R8G8B8 => TextureFormat::Bgra8Unorm,
        D3DFormat::R5G6B5 => TextureFormat::Rgba8Unorm,
        D3DFormat::A1R5G5B5 => TextureFormat::Rgba8Unorm,
        D3DFormat::A4R4G4B4 => TextureFormat::Rgba8Unorm,
        D3DFormat::A8 => TextureFormat::R8Unorm,
        D3DFormat::L8 => TextureFormat::R8Unorm,
        D3DFormat::A8L8 => TextureFormat::Rg8Unorm,
        D3DFormat::A4L4 => TextureFormat::Rg8Unorm,
        D3DFormat::V8U8 => TextureFormat::Rg8Unorm,
        D3DFormat::R8G8B8 => TextureFormat::Rgba8Unorm,
    }
}

/// Converts D3D9 optimized texture data into data that can be used by `wgpu`.
/// If no conversion is necessary, the vector will be given back. The texture
/// format will match the result of [`d3dformat_to_wgpu`].
fn convert_texture_format(info: &LevelTextureFormatInfo, data: Vec<u8>) -> Vec<u8> {
    match info.format {
        D3DFormat::DXT1 => data,
        D3DFormat::DXT3 => data,
        D3DFormat::A8R8G8B8 => data,
        D3DFormat::R5G6B5 => convert_color_depth(data, 5, 6, 5, 0),
        D3DFormat::A1R5G5B5 => convert_color_depth(data, 5, 5, 5, 1),
        D3DFormat::A4R4G4B4 => convert_color_depth(data, 4, 4, 4, 4),
        D3DFormat::A8 => data,
        D3DFormat::L8 => data,
        D3DFormat::A8L8 => data,
        D3DFormat::A4L4 => data,
        D3DFormat::V8U8 => data,
        D3DFormat::R8G8B8 => {
            // Convert to Rgba8Unorm
            data.into_iter()
                .tuples()
                .flat_map(|(b, g, r)| [r, g, b, 255])
                .collect()
        }
    }
}

/// Converts a specificied D3D9 BGRA 16-bit color depth uncompressed texture to
/// a roughly equivalent 32-bit RGBA counterpart (what a mouthful).
#[inline(always)]
fn convert_color_depth(
    data16: Vec<u8>,
    r_bits: u16,
    g_bits: u16,
    b_bits: u16,
    a_bits: u16,
) -> Vec<u8> {
    debug_assert_eq!(r_bits + g_bits + b_bits + a_bits, 16);

    data16
        .into_iter()
        .tuples()
        .flat_map(|(low, high)| {
            let value = u16::from_le_bytes([low, high]);

            let a_offset = r_bits + g_bits + b_bits;
            let r_offset = g_bits + b_bits;
            let g_offset = b_bits;
            let b_offset = 0;
            [
                color_to_8bits(value, r_offset, r_bits),
                color_to_8bits(value, g_offset, g_bits),
                color_to_8bits(value, b_offset, b_bits),
                if a_bits == 0 {
                    255
                } else {
                    color_to_8bits(value, a_offset, a_bits)
                },
            ]
        })
        .collect()
}

/// Converts a <8 bit color component to an 8-bit equivalent.
#[inline(always)]
fn color_to_8bits(pixel_value: u16, shift: u16, depth: u16) -> u8 {
    debug_assert!(0 < depth && depth < 8);

    // Extract the component value using bitops magic
    let value = (pixel_value >> shift) & ((1u16 << depth) - 1);

    // Perform a basic bitshift to extend the value to an 8-bit depth
    let mut result = value << (8 - depth);

    // We could end here, but it'd cause some inaccuracies, when for example converting a 5-bit
    // component - the highest value of 31 wouldn't translate to 255, but to 248.
    //
    // This happens because the newly created bottom 3 bits are initialized with zero. We need to
    // scale them proportionally to the source value. For example, with a 5-bit component,
    // color values [0; 31] get scaled to [0; 7] (as we are filling 3 bits).
    //
    // This way, a source value of 0 will still be translated to 0, and a value of 31 will be
    // translated to 255.
    //
    // A more generalized version of this would be scaling [0; 2**depth] to [0; 2**(8-depth)].
    // The expression below is basically a simplified version of this
    result |= value >> (depth + depth - 8);

    result as u8
}
