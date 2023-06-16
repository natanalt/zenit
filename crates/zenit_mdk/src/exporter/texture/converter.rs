use anyhow::ensure;
use byteorder::{WriteBytesExt, LE};
use itertools::Itertools;
use std::io::Write;
use zenit_lvl::game::D3DFormat;
use zenit_utils::{ok, AnyResult};

pub trait TextureConverter {
    /// Encodes an image into the specified writer.
    ///
    /// It is recommended that the writer be a buffered writer, as the exporters are likely to issue
    /// single byte writes to the writer.
    fn write_texture(
        &self,
        writer: &mut dyn Write,
        rgba: &[u8],
        width: u16,
        height: u16,
    ) -> AnyResult;

    fn size_after_conversion(&self, width: u16, height: u16) -> usize;
}

pub fn create_converter(format: D3DFormat) -> Box<dyn TextureConverter> {
    use D3DFormat::*;
    match format {
        DXT1 => Box::new(DXT1Exporter),
        DXT3 => Box::new(DXT3Exporter),
        A8R8G8B8 => todo!(),
        R5G6B5 => Box::new(R5G6B5Exporter),
        A1R5G5B5 => todo!(),
        A4R4G4B4 => todo!(),
        A8 => todo!(),
        L8 => todo!(),
        A8L8 => todo!(),
        A4L4 => todo!(),
        V8U8 => todo!(),
        R8G8B8 => Box::new(R8G8B8Exporter),
    }
}

// TODO: figure out why DXT1 textures weigh so much
pub struct DXT1Exporter;
impl TextureConverter for DXT1Exporter {
    fn write_texture(
        &self,
        writer: &mut dyn Write,
        rgba: &[u8],
        width: u16,
        height: u16,
    ) -> AnyResult {
        let format = texpresso::Format::Bc1;
        let size = format.compressed_size(width as usize, height as usize);
        let mut output = vec![0u8; size];

        format.compress(
            rgba,
            width as usize,
            height as usize,
            texpresso::Params {
                algorithm: texpresso::Algorithm::IterativeClusterFit,
                ..Default::default()
            },
            &mut output,
        );

        writer.write_all(&output)?;

        ok()
    }

    fn size_after_conversion(&self, width: u16, height: u16) -> usize {
        // Every 4x4 block of pixels is packed into 8 bytes
        width as usize * height as usize / 16 * 8
    }
}

pub struct DXT3Exporter;
impl TextureConverter for DXT3Exporter {
    fn write_texture(
        &self,
        writer: &mut dyn Write,
        rgba: &[u8],
        width: u16,
        height: u16,
    ) -> AnyResult {
        let format = texpresso::Format::Bc2;
        let size = format.compressed_size(width as usize, height as usize);
        let mut output = vec![0u8; size];

        format.compress(
            rgba,
            width as usize,
            height as usize,
            texpresso::Params {
                algorithm: texpresso::Algorithm::IterativeClusterFit,
                ..Default::default()
            },
            &mut output,
        );

        writer.write_all(&output)?;

        ok()
    }

    fn size_after_conversion(&self, width: u16, height: u16) -> usize {
        // Every 4x4 block of pixels is packed into 16 bytes
        width as usize * height as usize / 16 * 16
    }
}

/// R5G6B5 exporter. Clamps 8-bit channels by effectively rounding down.
pub struct R5G6B5Exporter;
impl TextureConverter for R5G6B5Exporter {
    fn write_texture(
        &self,
        writer: &mut dyn Write,
        rgba: &[u8],
        width: u16,
        height: u16,
    ) -> AnyResult {
        debug_assert_eq!(width as usize * height as usize * 4, rgba.len());
        for (r, g, b, a) in rgba.iter().cloned().tuples() {
            ensure!(a == 255, "the texture isn't fully transparent");
            let clamped_r = (r >> 3) as u16;
            let clamped_g = (g >> 2) as u16;
            let clamped_b = (b >> 3) as u16;
            let combined = clamped_r << 11 | clamped_g << 6 | clamped_b << 0;
            writer.write_u16::<LE>(combined)?;
        }
        ok()
    }

    fn size_after_conversion(&self, width: u16, height: u16) -> usize {
        // 2 bytes per pixel
        width as usize * height as usize * 2
    }
}

pub struct R8G8B8Exporter;
impl TextureConverter for R8G8B8Exporter {
    fn write_texture(
        &self,
        writer: &mut dyn Write,
        rgba: &[u8],
        width: u16,
        height: u16,
    ) -> AnyResult {
        debug_assert_eq!(width as usize * height as usize * 4, rgba.len());
        for (r, g, b, a) in rgba.iter().cloned().tuples() {
            ensure!(a == 255, "the texture isn't fully transparent");
            // Little endian 0xRRGGBB -> BB GG RR in memory
            writer.write_u8(b)?;
            writer.write_u8(g)?;
            writer.write_u8(r)?;
        }
        ok()
    }

    fn size_after_conversion(&self, width: u16, height: u16) -> usize {
        // 3 bytes per pixel
        width as usize * height as usize * 3
    }
}
