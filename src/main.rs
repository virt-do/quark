use clap::StructOpt;
use cli::{Cli, Result};

mod cli;

fn main() -> Result<()> {
    let cli: Cli = Cli::parse();

    cli.command().handler()?;

    Ok(())
}
