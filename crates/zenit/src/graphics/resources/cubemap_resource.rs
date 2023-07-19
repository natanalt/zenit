use crate::graphics::Renderer;
use glam::UVec2;
use std::sync::Arc;
use wgpu::TextureFormat;
use zenit_lvl::game::D3DFormat;
use zenit_proc::ext_repr;
use zenit_utils::ArcPoolHandle;

/// Handle to a 2D, 6 layer texture - a cubemap.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CubemapHandle(pub(in crate::graphics) ArcPoolHandle);

pub struct CubemapResource {
    pub label: String,
    pub handle: wgpu::Texture,
    pub format: wgpu::TextureFormat,
    /// The resource's primary texture view. Defined as an Arc to allow copying it to the render system.
    pub view: Arc<wgpu::TextureView>,
    pub unfiltered: bool,
    pub d3d_format: Option<D3DFormat>,
}

impl CubemapResource {
    pub fn new(r: &mut Renderer, desc: &CubemapDescriptor) -> Self {
        // TODO: perhaps the power of 2 texture size limitation could be lifted
        debug_assert!(
            desc.size.x.is_power_of_two() && desc.size.y.is_power_of_two(),
            "Texture dimensions must be powers of 2"
        );

        let handle = r.dc.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(&desc.name),
            size: wgpu::Extent3d {
                width: desc.size.x,
                height: desc.size.y,
                depth_or_array_layers: 6,
            },
            mip_level_count: desc.mip_levels,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: desc.format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[desc.format],
        });

        let view = Arc::new(handle.create_view(&wgpu::TextureViewDescriptor {
            label: Some(&desc.name),
            format: Some(desc.format),
            dimension: Some(wgpu::TextureViewDimension::D2),
            aspect: wgpu::TextureAspect::All,
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: None,
        }));

        Self {
            label: desc.name.clone(),
            format: desc.format,
            handle,
            view,
            unfiltered: desc.unfiltered,
            d3d_format: desc.d3d_format,
        }
    }
}

/// Cubemap functions
impl Renderer {
    pub fn write_cubemap(&mut self, desc: &CubemapWriteDescriptor) {
        let dc = &self.dc;
        let cubemap = self.cubemaps.get(desc.handle);
        let format = cubemap.format;
        let block_width = format.block_dimensions().0 as u32;
        let block_height = format.block_dimensions().1 as u32;
        let block_bytes = format
            .block_size(None)
            .expect("block_size failed, invalid 2D texture format?")
            as u32;

        // Texture size adjusted for the mipmap level
        let texture_width = (cubemap.handle.width() >> desc.mip_level).max(block_width);
        let texture_height = (cubemap.handle.height() >> desc.mip_level).max(block_height);

        dc.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &cubemap.handle,
                mip_level: desc.mip_level,
                origin: wgpu::Origin3d {
                    x: 0,
                    y: 0,
                    z: desc.face as u32,
                },
                aspect: wgpu::TextureAspect::All,
            },
            desc.data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(block_bytes * (texture_width / block_width)),
                rows_per_image: Some(texture_height / block_height),
            },
            wgpu::Extent3d {
                width: texture_width,
                height: texture_height,
                depth_or_array_layers: 1,
            },
        );
    }
}

/// Defines each face of a cubemap. The `u8` discriminator specifies the appropriate texture layer
/// index for the cubemap.
#[ext_repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CubemapFace {
    /// +X direction
    Right = 0,
    /// -X direction
    Left = 1,
    /// +Y direction
    Up = 2,
    /// -Y direction
    Down = 3,
    /// +Z direction
    Forward = 4,
    /// -Z direction
    Backward = 5,
}

pub struct CubemapDescriptor {
    pub name: String,
    pub size: UVec2,
    pub mip_levels: u32,
    pub format: TextureFormat,
    pub d3d_format: Option<D3DFormat>,
    pub unfiltered: bool,
}

pub struct CubemapWriteDescriptor<'a> {
    pub handle: &'a CubemapHandle,
    pub mip_level: u32,
    pub face: CubemapFace,
    pub data: &'a [u8],
}
