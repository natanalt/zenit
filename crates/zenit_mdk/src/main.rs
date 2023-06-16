use clap::Parser;
use zenit_utils::{ok, AnyResult};

fn main() -> AnyResult {
    let cli = zenit_mdk::Cli::parse_from(wild::args());
    zenit_mdk::run(cli)?;
    ok()
}
