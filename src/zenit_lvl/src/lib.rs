use std::ffi::CString;
use zenit_proc::ext_repr;

pub use zenit_lvl_core::*;

#[ext_repr(u32)]
#[derive(Debug, Clone)]
pub enum FormatKind {
    A,
}
#[ext_repr(u32)]
#[derive(Debug, Clone)]
pub enum TextureKind {
    A,
}

zenit_proc::define_node_type! {
    "ucfb" as LevelData {
        "scr_" -> scripts: Vec<LevelScript> {
            "NAME" -> name: CString,
            "INFO" -> info: u8,
            "BODY" -> data: Box<Vec::<u8>>,
        }

        "tex_" -> textures: Vec<LevelTexture> {
            "NAME" -> name: CString,
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
                        },
                        "BODY" -> data: Box<Vec::<u8>>,
                    }
                }
            }
        }
    }
}