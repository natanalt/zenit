use super::RenderContext;
use glam::*;
use std::num::NonZeroU32;
use zenit_proc::ext_repr;

pub struct TextureSource {
    pub label: Option<String>,
    pub handle: wgpu::Texture,
    pub format: wgpu::TextureFormat,
    pub usages: wgpu::TextureUsages,
    pub mip_count: u32,
    pub size: UVec3,
}

impl TextureSource {
    pub fn new(ctx: &RenderContext, descriptor: &wgpu::TextureDescriptor) -> Self {
        Self {
            label: descriptor.label.map(String::from),
            handle: ctx.device.create_texture(descriptor),
            format: descriptor.format,
            usages: descriptor.usage,
            mip_count: descriptor.mip_level_count,
            size: UVec3::new(
                descriptor.size.width,
                descriptor.size.height,
                descriptor.size.depth_or_array_layers,
            ),
        }
    }

    pub fn into_2d(self) -> Texture2D {
        // TODO: add dimension checks (based on init TextureDescriptor)?
        Texture2D {
            view: self.handle.create_view(&wgpu::TextureViewDescriptor {
                label: None,
                format: Some(self.format),
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: 0,
                mip_level_count: NonZeroU32::new(self.mip_count),
                base_array_layer: 0,
                array_layer_count: NonZeroU32::new(1),
            }),
            source: self,
        }
    }

    pub fn into_cubemap(self) -> TextureCubemap {
        assert!(self.size.z == 6, "Invalid texture sizing for a cubemap!");
        TextureCubemap {
            view: self.handle.create_view(&wgpu::TextureViewDescriptor {
                label: None,
                format: Some(self.format),
                dimension: Some(wgpu::TextureViewDimension::Cube),
                aspect: wgpu::TextureAspect::All,
                base_mip_level: 0,
                mip_level_count: NonZeroU32::new(self.mip_count),
                base_array_layer: 0,
                array_layer_count: NonZeroU32::new(6),
            }),
            source: self,
        }
    }
}

pub struct Texture2D {
    pub source: TextureSource,
    pub view: wgpu::TextureView,
}

impl Texture2D {
    pub fn from_rgba8(ctx: &RenderContext, size: UVec2, data: &[u8]) -> Self {
        let result = TextureSource::new(
            ctx,
            &wgpu::TextureDescriptor {
                label: None,
                size: wgpu::Extent3d {
                    width: size.x,
                    height: size.y,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
            },
        )
        .into_2d();
        result.write_to_mip(ctx, 0, data);
        result
    }

    /// Schedules a write to the specified texture mip. Passed data is supposed
    /// to be in the correct texture format, and is meant to cover the entire
    /// mip surface.
    ///
    /// **Note:** the texture cannot be compressed, otherwise the bytes per row
    /// will be miscalculated.
    pub fn write_to_mip(&self, ctx: &RenderContext, mip: u32, data: &[u8]) {
        let downscale_factor = 2u32.pow(mip);
        let size = self.source.size.xy() / downscale_factor;

        ctx.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.source.handle,
                mip_level: mip,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(
                    self.source.format.describe().block_size as u32 * size.x,
                ),
                rows_per_image: NonZeroU32::new(size.y),
            },
            wgpu::Extent3d {
                width: size.x,
                height: size.y,
                depth_or_array_layers: 1,
            },
        );
    }
}

pub struct TextureCubemap {
    pub source: TextureSource,
    pub view: wgpu::TextureView,
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[ext_repr(u32)]
pub enum CubeFace {
    PositiveX = 0,
    NegativeX = 1,
    PositiveY = 2,
    NegativeY = 3,
    PositiveZ = 4,
    NegativeZ = 5,
}

impl TextureCubemap {
    /// Schedules write to a mip of a specific face of this cubemap. Otherwise
    /// it's similar to [`Texture2D::write_to_mip`]
    pub fn write_to_mip(&self, ctx: &RenderContext, face: CubeFace, mip: u32, data: &[u8]) {
        let downscale_factor = 2u32.pow(mip);
        let size = self.source.size.xy() / downscale_factor;

        ctx.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.source.handle,
                mip_level: mip,
                origin: wgpu::Origin3d {
                    x: 0,
                    y: 0,
                    z: face.into(),
                },
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(
                    self.source.format.describe().block_size as u32 * size.x,
                ),
                rows_per_image: NonZeroU32::new(size.y),
            },
            wgpu::Extent3d {
                width: size.x,
                height: size.y,
                depth_or_array_layers: 1,
            },
        );
    }
}
