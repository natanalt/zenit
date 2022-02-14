use zenit_proc::ext_repr;

#[ext_repr(u32)]
#[derive(Debug, Clone)]
pub enum FormatKind { A }
#[ext_repr(u32)]
#[derive(Debug, Clone)]
pub enum TextureKind { A }

use crate as zenit_lvl;
crate::define_node_type! {
    "ucfb" as LevelData {
        "scr_" -> scripts: Vec<LevelScript> {
            "NAME" -> name: String,
            "INFO" -> info: u8,
            "BODY" -> data: Vec<u8>,
        }
        
        "tex_" -> textures: Vec<LevelTexture> {
            "NAME" -> name: String,
            "FMT_" -> formats: Vec<TextureFormat> {
                "INFO" -> info: TextureInfo {
                    format: FormatKind as u32,
                    width: u16,
                    height: u16,
                    unknown: u16,
                    mipmaps: u16,
                    kind: TextureKind as u32,
                }
                
                "FACE" -> faces: Vec<TextureFace> {
                    "LVL_" -> mipmaps: Vec<TextureMipmap> {
                        "INFO" -> info: MipmapInfo {
                            level: u32,
                            size: u32,
                        }
                        "BODY" -> data: Vec<u8>,
                    }
                }
            }
        }
    }
}
