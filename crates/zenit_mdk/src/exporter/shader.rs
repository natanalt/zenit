use serde::Deserialize;
use std::{ffi::CString, fs, path::PathBuf};
use zenit_lvl::zext::LevelWgslShader;
use zenit_utils::AnyResult;

#[derive(Debug, Deserialize)]
pub struct ShaderSpecification {
    pub name: String,
    pub file: PathBuf,
}

impl ShaderSpecification {
    pub fn export(self, include: &str) -> AnyResult<LevelWgslShader> {
        Ok(LevelWgslShader {
            name: CString::new(self.name)?,
            code: CString::new(format!(
                "// Shared WGSL files begin\n\n{include}\n// Shader code begin\n\n{}",
                fs::read_to_string(self.file)?
            ))?,
        })
    }
}
