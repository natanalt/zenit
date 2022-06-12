use self::texture::TextureAsset;

pub mod texture;

pub struct AssetManager {
    pub textures: Vec<TextureAsset>,
}
