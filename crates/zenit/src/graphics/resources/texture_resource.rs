use crate::graphics::Renderer;
use glam::*;
use std::sync::Arc;
use wgpu::TextureFormat;
use zenit_utils::ArcPoolHandle;

/// Handle to a 2D, 1 layer texture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextureHandle(pub(in crate::graphics) ArcPoolHandle);

pub struct TextureResource {
    pub label: String,
    pub handle: wgpu::Texture,
    pub format: wgpu::TextureFormat,
    /// The resource's primary texture view. Defined as an Arc to allow copying it to the render system.
    pub view: Arc<wgpu::TextureView>,
    pub unfiltered: bool,
}

impl TextureResource {
    pub fn new(r: &mut Renderer, desc: &TextureDescriptor) -> Self {
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
                depth_or_array_layers: 1,
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
        }
    }
}

/// 2D texture functions
impl Renderer {
    /// Schedules a 2D texture write.
    ///
    /// The underlying data is expected to be in the texture's format. Its layout is assumed to
    /// be packed. Look at this function's code for the specific algorithm that tries to work
    /// generically with every texture format. RGBA8 and BCx writes should always work, other
    /// formats aren't tested.
    ///
    /// `write_2d_texture` tries to provide an interface that will *usually* be enough. For more
    /// complicated cases you may want to use [`wgpu`] functionality directly c:
    pub fn write_texture(&self, desc: &TextureWriteDescriptor) {
        let dc = &self.dc;
        let texture = self.textures.get(desc.handle);

        let format = texture.format;
        let block_width = format.block_dimensions().0 as u32;
        let block_height = format.block_dimensions().1 as u32;
        let block_bytes = format
            .block_size(None)
            .expect("block_size failed, invalid 2D texture format?")
            as u32;

        // Texture size adjusted for the mipmap level
        let texture_width = (texture.handle.width() >> desc.mip_level).max(block_width);
        let texture_height = (texture.handle.height() >> desc.mip_level).max(block_height);

        dc.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture.handle,
                mip_level: desc.mip_level,
                origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            desc.data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(block_bytes * (texture_width / block_width)),
                rows_per_image: None, // Not a multilayered image
            },
            wgpu::Extent3d {
                width: texture_width,
                height: texture_height,
                depth_or_array_layers: 1,
            },
        );
    }
}

pub struct TextureDescriptor {
    pub name: String,
    pub size: UVec2,
    pub mip_levels: u32,
    pub format: TextureFormat,
    pub unfiltered: bool,
}

pub struct TextureWriteDescriptor<'a> {
    pub handle: &'a TextureHandle,
    pub mip_level: u32,
    pub data: &'a [u8],
}
