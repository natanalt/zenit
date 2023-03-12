use crate::render::DeviceContext;
use include_dir::{include_dir, Dir};
use log::*;
use rustc_hash::FxHashMap;
use std::sync::Arc;
use thiserror::Error;
use wgpu::*;
use zenit_utils::AnyResult;

// TODO: move shader preprocessing to build time

static SHADER_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/render/shaders");

/// Stores compiled shader modules.
///
/// ## Zenit Shader Preprocessor
/// A simple utility, currently only supporting includes.
///
/// Each preprocessor directive takes up its own line, starting and ending with square brackets.
/// Note, that the preprocessor does *not* follow block comments, and as such including a file
/// inside a block comment is possible. Since WGSL block comments can nest, including files
/// that have block comments inside a block comment itself shouldn't be an issue in all valid
/// cases.
///
/// Preprocessor directives can contain leading and trailing whitespace, as each processed line
/// is trimmed during the check.
///
/// Preproessing directives are case sensitive.
///
/// ### \[include ...]
/// Other files (relative to, and within the shader directory) can be included into the preprocessed
/// shader as follows:
///
/// ```wgsl
/// // Includes camera.inc.wgsl, which has the CameraBuffer struct definition
/// [include camera.inc.wgsl]
///
/// @group(0) @binding(0)
/// var<uniform> camera: CameraBuffer;
/// ```
/// The path name cannot contain spaces
///
/// **Note:** Recursive inclusions are not handled correctly and will result in a recursive loop,
/// likely finishing with a stack overflow.
///
pub struct Shader {
    pub path: String,
    pub module: wgpu::ShaderModule,
}

pub struct ShaderManager {
    shader_cache: FxHashMap<String, Arc<Shader>>,
}

impl ShaderManager {
    pub fn new() -> Self {
        Self {
            shader_cache: FxHashMap::default(),
        }
    }

    pub fn load_shader(&mut self, dc: &DeviceContext, path: &str) -> AnyResult<Arc<Shader>> {
        if let Some(cached) = self.shader_cache.get(path) {
            Ok(cached.clone())
        } else {
            let shader = Arc::new(Shader {
                module: dc.device.create_shader_module(ShaderModuleDescriptor {
                    label: Some(path),
                    source: ShaderSource::Wgsl(preprocess_shader(path)?.into()),
                }),
                path: path.to_string(),
            });

            self.shader_cache.insert(path.to_string(), shader.clone());
            Ok(shader)
        }
    }

    pub fn cleanup(&mut self) {
        self.shader_cache.retain(|_, s| Arc::strong_count(s) == 1);
    }
}

#[derive(Debug, Error)]
pub enum PreprocessingError {
    #[error("Shader file `{0}` not found")]
    NotFound(String),
    #[error("Shader file `{0}` is incorrectly encoded (not UTF-8)")]
    InvalidEncoding(String),
    #[error("Empty directive @ {0}:{1}")]
    EmptyDirective(String, usize),
    #[error("Unknown directive `{2}` @ {0}:{1}")]
    UnknownDirective(String, usize, String),
    #[error("Expected an include path @ {0}:{1}")]
    ExpectedIncludePath(String, usize),
    #[error("Including a non-existent file `{2}` @ {0}:{1}")]
    IncludedNotFound(String, usize, String),
    #[error("An error occurred while preprocessing included file @ {0}:{1}: {2}")]
    IncludePreprocessorError(String, usize, Box<PreprocessingError>),
}

/// Preprocesses a shader at a specified path. Returns the preprocessed string.
///
/// See [`ShaderResource`] documentation for info on the preprocessor format.
pub fn preprocess_shader(path: &str) -> Result<String, PreprocessingError> {
    trace!("Preprocessing shader `{path}`...");

    let source = SHADER_DIR
        .get_file(path)
        .ok_or_else(|| PreprocessingError::NotFound(path.to_string()))?
        .contents_utf8()
        .ok_or_else(|| PreprocessingError::NotFound(path.to_string()))?;

    let mut result = String::with_capacity(source.len());

    for (line_index, line) in source.lines().enumerate() {
        let trimmed = line.trim();

        // Cutting off empty lines shouldn't be an issue
        if trimmed.is_empty() {
            continue;
        }

        let first = trimmed.bytes().next().unwrap();
        let last = trimmed.bytes().last().unwrap();

        if first == b'[' && last == b']' {
            let directive = &trimmed[1..trimmed.len() - 1];
            if directive.is_empty() {
                return Err(PreprocessingError::EmptyDirective(
                    path.to_string(),
                    line_index + 1,
                ));
            }

            let mut split = directive.split(' ');
            let directive_name = split.next().unwrap();

            match directive_name {
                "include" => {
                    let file_name = split.next().ok_or(PreprocessingError::ExpectedIncludePath(
                        path.to_string(),
                        line_index + 1,
                    ))?;

                    let preprocessed = preprocess_shader(file_name).map_err(|error| {
                        PreprocessingError::IncludePreprocessorError(
                            path.to_string(),
                            line_index + 1,
                            Box::new(error),
                        )
                    })?;

                    result.push_str(&preprocessed);
                    result.push('\n');
                }
                _ => {
                    return Err(PreprocessingError::UnknownDirective(
                        path.to_string(),
                        line_index + 1,
                        directive_name.to_string(),
                    ))
                }
            }
        } else {
            result.push_str(line);
            result.push('\n');
        }
    }

    Ok(result)
}
