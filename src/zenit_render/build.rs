use std::{path::{PathBuf, Path}, env, io::{self, BufReader, BufRead}, fs::{File, self}, collections::HashMap, ffi::OsString, process};
use derive_builder::Builder;
use which::which;

pub struct GlslToolchain {
    /// Actual name: `glslc`
    compiler: PathBuf,
    /// Actual name: `spirv-link`
    linker: PathBuf,
}

impl GlslToolchain {
    pub fn find() -> Self {
        Self {
            compiler: Self::search_single("glslc")
                .or(Self::search_single("glslc.exe"))
                .expect("glslc not found"),
            linker: Self::search_single("spirv-link")
                .or(Self::search_single("spirv-link.exe"))
                .expect("spirv-link not found"),
        }
    }

    fn search_single(name: &str) -> Option<PathBuf> {
        if let Ok(path) = which(name) {
            return Some(path);
        }
        let vulkan_root = PathBuf::from(env::var("VULKAN_SDK").ok()?);
        let vulkan_bin = vulkan_root.join("bin");
        let file = vulkan_bin.join(name);
        file.is_file().then(|| file)
    }
}

#[derive(Debug, Clone)]
pub struct ShaderSource {
    pub metadata: String,
    pub shared: String,
    pub vertex: String,
    pub fragment: String,
}

impl ShaderSource {
    /// .shader file reader with numerous limitations
    pub fn parse(file: &Path) -> io::Result<Self> {
        let reader = BufReader::new(File::open(file)?);
        
        let mut metadata = String::new();
        let mut shared = String::new();
        let mut vertex = String::new();
        let mut fragment = String::new();
        
        #[derive(PartialEq, Eq, Hash, Clone, Copy)]
        enum State {
            Metadata,
            Shared,
            Vertex,
            Fragment,
        }

        let mut targets = HashMap::from([
            (State::Metadata, &mut metadata),
            (State::Shared, &mut shared),
            (State::Vertex, &mut vertex),
            (State::Fragment, &mut fragment),
        ]);

        let transitions = HashMap::from([
            ("shared {", State::Shared),
            ("vertex {", State::Vertex),
            ("fragment {", State::Fragment),
        ]);

        /////////////////////////////////////////

        let mut state = State::Metadata;
        let mut nested_brackets = 0;

        for line in reader.lines().map(Result::unwrap) {
            if line.starts_with("//") {
                continue;
            }

            let target = targets.get_mut(&state).unwrap();

            if state == State::Metadata {
                let new_state = transitions
                    .iter()
                    .find(|(k, _)| line == **k)
                    .map(|(_, v)| *v);

                if let Some(new_state) = new_state {
                    state = new_state;
                    nested_brackets = 1;
                } else {
                    target.push_str(&line);
                    target.push('\n');
                }
            } else {
                for c in line.chars() {
                    if c == '{' {
                        nested_brackets += 1;
                    } else if c == '}' {
                        nested_brackets -= 1;
                    }
                }

                if nested_brackets == 0 {
                    state = State::Metadata;
                } else {
                    target.push_str(&line);
                    target.push('\n');
                }
            }
        }

        /////////////////////////////////////////

        Ok(Self {
            metadata,
            shared,
            vertex,
            fragment,
        })
    }
}

pub struct CompilerInvocation<'t> {
    pub toolchain: Option<&'t GlslToolchain>,
    pub args: Vec<String>,
}

impl<'t> CompilerInvocation<'t> {
    pub fn builder() -> Self {
        Self { toolchain: None, args: vec![] }
    }

    pub fn toolchain(&mut self, toolchain: &'t GlslToolchain) -> &mut Self {
        self.toolchain = Some(toolchain);
        self
    }

    pub fn arg(&mut self, arg: impl AsRef<str>) -> &mut Self {
        self.args.push(arg.as_ref().to_owned());
        self
    }

    pub fn invoke(&self) -> io::Result<()> {
        let output = process::Command::new(&self.toolchain.unwrap().compiler)
            .arg("-O")
            .arg("-Iassets/shaders")
            .arg("-std=450core")
            .arg("--target-env=vulkan")
            .args(&self.args)
            .output()?;

        if !output.status.success() {
            println!("Compilation failure!");
            eprintln!("======================================");
            eprintln!("stdout:\n{}", String::from_utf8_lossy(&output.stdout));
            eprintln!("------------------------------------");
            eprintln!("stderr:\n{}", String::from_utf8_lossy(&output.stderr));
            eprintln!("======================================");
            process::exit(1);
        }

        Ok(())
    }
}

pub struct LinkerInvocation<'t> {
    pub toolchain: Option<&'t GlslToolchain>,
    pub args: Vec<String>,
}

impl<'t> LinkerInvocation<'t> {
    pub fn builder() -> Self {
        Self { toolchain: None, args: vec![] }
    }

    pub fn toolchain(&mut self, toolchain: &'t GlslToolchain) -> &mut Self {
        self.toolchain = Some(toolchain);
        self
    }

    pub fn arg(&mut self, arg: impl AsRef<str>) -> &mut Self {
        self.args.push(arg.as_ref().to_owned());
        self
    }

    pub fn invoke(&self) -> io::Result<()> {
        let output = process::Command::new(&self.toolchain.unwrap().linker)
            .args(&self.args)
            .output()?;

        if !output.status.success() {
            println!("Linking failure!");
            eprintln!("======================================");
            eprintln!("stdout:\n{}", String::from_utf8_lossy(&output.stdout));
            eprintln!("------------------------------------");
            eprintln!("stderr:\n{}", String::from_utf8_lossy(&output.stderr));
            eprintln!("======================================");
            process::exit(1);
        }

        Ok(())
    }
}

#[derive(Builder)]
pub struct ShaderCompilation<'t> {
    pub toolchain: &'t GlslToolchain,
    pub name: String,
    pub metadata: String,
    pub vertex_shader: String,
    pub fragment_shader: String, 
}

impl<'t> ShaderCompilation<'t> {
    pub fn builder() -> ShaderCompilationBuilder<'t> {
        ShaderCompilationBuilder::default()
    }

    pub fn invoke(&self) -> io::Result<()> {
        let path_vertex_glsl = out_dir_path(&format!("{}.vert.glsl", &self.name));
        let path_fragment_glsl = out_dir_path(&format!("{}.frag.glsl", &self.name));
        let path_vertex_spirv = out_dir_path(&format!("{}.vert.spv", &self.name));
        let path_fragment_spirv = out_dir_path(&format!("{}.frag.spv", &self.name));
        let path_spirv = out_dir_path(&format!("{}.spv", &self.name));
        let path_toml = out_dir_path(&format!("{}.toml", &self.name));

        write_out_file(&path_toml, self.metadata.as_bytes())?;
        write_out_file(&path_vertex_glsl, self.vertex_shader.as_bytes())?;
        write_out_file(&path_fragment_glsl, self.fragment_shader.as_bytes())?;

        println!("  : Compiling vertex shader...");
        CompilerInvocation::builder()
            .toolchain(self.toolchain)
            .arg("-DVERTEX")
            .arg("-fshader-stage=vertex")
            .arg(path_vertex_glsl.to_str().unwrap())
            .arg("-o")
            .arg(path_vertex_spirv.to_str().unwrap())
            .invoke()?;
        
        println!("  : Compiling fragment shader...");
        CompilerInvocation::builder()
            .toolchain(self.toolchain)
            .arg("-DFRAGMENT")
            .arg("-fshader-stage=fragment")
            .arg(path_fragment_glsl.to_str().unwrap())
            .arg("-o")
            .arg(path_fragment_spirv.to_str().unwrap())
            .invoke()?;
        
        println!("  : Linking...");
        LinkerInvocation::builder()
            .toolchain(self.toolchain)
            .arg(path_fragment_spirv.to_str().unwrap())
            .arg(path_vertex_spirv.to_str().unwrap())
            .arg("-o")
            .arg(path_spirv.to_str().unwrap())
            .invoke()?;

        Ok(())
    }
}

fn out_dir_path(path: impl AsRef<Path>) -> PathBuf {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    out_dir.join(path)
}

fn write_out_file(path: impl AsRef<Path>, contents: &[u8]) -> io::Result<PathBuf> {
    let final_path = out_dir_path(path);
    fs::write(&final_path, contents)?;
    Ok(final_path)
}

fn main() -> io::Result<()> {
    println!("cargo:rerun-if-changed=assets/shaders");

    let toolchain = GlslToolchain::find();

    for entry in fs::read_dir("assets/shaders")? {
        let entry = entry?;
        let path = entry.path();

        if !entry.file_type()?.is_file() {
            continue;
        }

        if path.extension() != Some(&OsString::from("shader")) {
            continue;
        }

        let name = path.file_name().unwrap().to_string_lossy();
        println!("Compiling shader `{}`...", &name);

        let source = ShaderSource::parse(&path)?;

        ShaderCompilation::builder()
            .toolchain(&toolchain)
            .name(name.to_string())
            .metadata(source.metadata)
            .vertex_shader(format!("{}\n{}", source.shared, source.vertex))
            .fragment_shader(format!("{}\n{}", source.shared, source.fragment))
            .build()
            .unwrap()
            .invoke()?;
    }

    Ok(())
}
