use std::{path::PathBuf, ffi::CString, fs};
use serde::Deserialize;
use zenit_lvl::zext::LevelWgslShader;
use zenit_utils::AnyResult;

#[derive(Debug, Deserialize)]
pub struct ShaderSpecification {
    pub name: String,
    pub file: PathBuf,
}

impl ShaderSpecification {
    pub fn export(self) -> AnyResult<LevelWgslShader> {
        Ok(LevelWgslShader {
            name: CString::new(self.name)?,
            code: CString::new(fs::read_to_string(self.file)?)?,
        })
    }
}
