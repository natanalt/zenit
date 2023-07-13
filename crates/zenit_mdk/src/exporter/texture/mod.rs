use anyhow::{bail, ensure};
use byteorder::{WriteBytesExt, LE};
use itertools::Itertools;
use serde::Deserialize;
use std::{ffi::CString, io::Cursor, path::PathBuf};
use zenit_lvl::game::{
    D3DFormat, LevelTexture, LevelTextureFace, LevelTextureFormat, LevelTextureFormatInfo,
    LevelTextureKind, LevelTextureMipmap, LevelTextureMipmapInfo, ZLevelTextureFiltering,
};
use zenit_utils::AnyResult;

pub mod converter;

#[derive(Debug, Deserialize)]
pub enum CubemapFaces {
    Repeated,
    Sheeted,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", content = "faces" /* This def is a bit hacky as it only applies to ::Cubemap */)]
pub enum TextureKind {
    Color,
    Cubemap(#[serde(rename = "faces")] CubemapFaces),
}

#[derive(Debug, Deserialize)]
pub struct TextureSpecification {
    pub name: String,
    pub file: PathBuf,
    pub formats: Vec<D3DFormat>,
    #[serde(flatten)]
    pub kind: TextureKind,
    pub mipmaps: Option<u16>,
    #[serde(default)]
    pub unfiltered: bool,
}

impl TextureSpecification {
    pub fn export(self) -> AnyResult<LevelTexture> {
        let TextureSpecification {
            name,
            file,
            formats,
            kind,
            mipmaps,
            unfiltered,
        } = self;
    
        let mut texture = LevelTexture {
            name: CString::new(name)?,
            formats: Vec::with_capacity(formats.len()),
            info: {
                // tex_:INFO has a dynamically sized format, so we just assemble it here manually
                let mut result = vec![];
                let mut cursor = Cursor::new(&mut result);
                cursor.write_u32::<LE>(formats.len().try_into()?)?;
                for &format in &formats {
                    cursor.write_u32::<LE>(format.into())?;
                }
                result
            },
        };
    
        let image = image::open(file)?.into_rgba8();
        ensure!(image.width() <= i16::MAX as u32, "image too large");
        ensure!(image.height() <= i16::MAX as u32, "image too large");
        let width = image.width() as u16;
        let height = image.height() as u16;
    
        let mipmaps = match mipmaps {
            Some(user_amount) => user_amount,
            None => {
                // plus 1 cause we're also counting power of 0
                width.max(height).ilog2() as u16 + 1
            }
        };
    
        for format in formats {
            let generated_mipmaps = generate_mipmaps(
                &image.as_raw(),
                width as usize,
                height as usize,
                mipmaps as usize,
            );
    
            use CubemapFaces::*;
            use TextureKind::*;
            texture.formats.push(LevelTextureFormat {
                info: LevelTextureFormatInfo {
                    format,
                    width,
                    height,
                    unk_0x08: 1, // ??
                    mipmaps,
                    kind: match &kind {
                        Color => LevelTextureKind::D2,
                        Cubemap(_) => LevelTextureKind::Cubemap,
                    },
                },
                faces: match &kind {
                    Color => vec![export_face(generated_mipmaps, format)?],
                    Cubemap(cubemap_faces) => match cubemap_faces {
                        Repeated => vec![export_face(generated_mipmaps, format)?; 6],
                        Sheeted => bail!("cubemap sheets aren't supported yet"),
                    },
                },
                unfiltered: if unfiltered {
                    vec![ZLevelTextureFiltering {}]
                } else {
                    vec![]
                },
            });
        }
    
        Ok(texture)
    }
}

/// Shrinks the texture by a factor of 2, filtering the pixel by using the average of 4x4 blocks.
/// In other words, this generates another mip level in the chain.
fn downscale_texture(rgba: &[u8], width: usize, height: usize) -> Vec<u8> {
    // This could be potentially heavily optimized with SIMD ops, unless the compiler
    // figures it out automatically
    //
    // I could look into that someday :3

    debug_assert!(width.is_power_of_two() && height.is_power_of_two());
    debug_assert!(rgba.len() == width * height * 4);
    if width == 1 || height == 1 {
        return Vec::from(rgba);
    }

    let mut result = Vec::with_capacity(rgba.len() / 4);

    for two_rows in rgba.chunks(width * 4 * 2) {
        let (upper, lower) = two_rows.split_at(two_rows.len() / 2);

        let upper_doubles = upper.iter().map(|&byte| byte as u16).tuples();
        let lower_doubles = lower.iter().map(|&byte| byte as u16).tuples();

        for ((r1, g1, b1, a1, r2, g2, b2, a2), (r3, g3, b3, a3, r4, g4, b4, a4)) in
            upper_doubles.zip(lower_doubles)
        {
            result.push(((r1 + r2 + r3 + r4) / 4) as u8);
            result.push(((g1 + g2 + g3 + g4) / 4) as u8);
            result.push(((b1 + b2 + b3 + b4) / 4) as u8);
            result.push(((a1 + a2 + a3 + a4) / 4) as u8);
        }
    }

    result
}

fn generate_mipmaps(
    rgba: &[u8],
    width: usize,
    height: usize,
    mipmap_count: usize,
) -> Vec<(u16, u16, Vec<u8>)> {
    let mut mipmaps = Vec::with_capacity(mipmap_count);

    let mut current_rgba = Vec::from(rgba);
    let mut current_width = width;
    let mut current_height = height;
    for _ in 0..mipmap_count {
        mipmaps.push((
            current_width.try_into().unwrap(),
            current_height.try_into().unwrap(),
            current_rgba.clone(),
        ));

        // Only generate further downsamples if we aren't at either dimension of 1.
        // We keep on generating mips even if they're dupes, as we were asked for a
        // specific amount of mips, and we're a good obedient girl :3
        if current_width != 1 && current_height != 1 {
            current_rgba = downscale_texture(&current_rgba, current_width, current_height);
            current_width /= 2;
            current_height /= 2;
        }
    }

    mipmaps
}

fn export_face(
    rgba_mipmaps: Vec<(u16, u16, Vec<u8>)>,
    format: D3DFormat,
) -> AnyResult<LevelTextureFace> {
    let tc = converter::create_converter(format);

    Ok(LevelTextureFace {
        mipmaps: rgba_mipmaps
            .into_iter()
            .enumerate()
            .map(|(level, (width, height, rgba))| {
                let mut body = Vec::with_capacity(tc.size_after_conversion(width, height));
                let mut cursor = Cursor::new(&mut body);

                tc.write_texture(&mut cursor, &rgba, width, height)?;
                Ok(LevelTextureMipmap {
                    info: LevelTextureMipmapInfo {
                        mip_level: level.try_into()?,
                        body_size: body.len().try_into()?,
                    },
                    body: body.into(),
                })
            })
            .collect::<AnyResult<Vec<_>>>()?,
    })
}
