use clap::Args;
use std::{
    fs::{self, File},
    io::{Read, Seek},
    path::PathBuf,
};
use zenit_utils::{ok, AnyResult};

#[derive(Args)]
pub struct MergeCommand {
    /// Output file
    #[clap(long, short = 'o')]
    pub output: PathBuf,
    /// Individual input files to merge
    pub files: Vec<PathBuf>,
}

impl crate::Command for MergeCommand {
    fn run(self) -> AnyResult {
        println!("Merging files into {}...", self.output.display());

        let _output = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&self.output)?;

        for input_path in &self.files {
            println!("  Merging {}...", input_path.display());

            let mut file = File::open(input_path)?;

            // Validate and skip the header
            let mut header = [0u8; 4];
            file.read_exact(&mut header)?;
            file.seek(std::io::SeekFrom::Current(4))?;
            if &header != b"ucfb" {
                println!("    Warning: this file doesn't contain a valid header. Skipping...");
                continue;
            }

            todo!()
        }

        ok()
    }
}
