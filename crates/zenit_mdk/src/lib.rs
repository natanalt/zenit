//! The Zenit MDK - mod development kit
//!
//! Currently it consists of a command line utility capable of generating Zenit/BF2 compatible
//! data files.

use clap::{Parser, Subcommand};
use commands::{
    append::AppendCommand, build::BuildCommand, export::ExportCommand, merge::MergeCommand,
};
use zenit_utils::{ok, AnyResult};

pub mod commands;
pub mod exporter;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: CliCommand,
}

#[derive(Subcommand)]
pub enum CliCommand {
    /// Builds a complete data file from specification
    Build(BuildCommand),
    /// Merges several data files together
    Merge(MergeCommand),
    /// Exports specified resources from a data file
    #[command(subcommand)]
    Export(ExportCommand),
    /// Appends new data into a data file
    #[command(subcommand)]
    Append(AppendCommand),
}

pub trait Command {
    fn run(self) -> AnyResult;
}

/// Runs `zenit_mdk` as if it was ran from the command line.
///
/// This function is provided to allow invocation of the MDK tools from library
/// builds.
pub fn run(cli: Cli) -> AnyResult {
    match cli.command {
        CliCommand::Build(c) => c.run()?,
        CliCommand::Merge(c) => c.run()?,
        CliCommand::Export(c) => c.run()?,
        CliCommand::Append(c) => c.run()?,
    }
    ok()
}
