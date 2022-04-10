use clap::StructOpt;
use cli::{Cli, Result};
use log::{log_enabled, Level, LevelFilter};
use std::io::Write;

mod cli;
mod config;
mod helper;

fn main() -> Result<()> {
    let cli: Cli = Cli::parse();

    let mut builder = env_logger::Builder::new();
    let logger = builder
        .filter_level(match cli.verbose {
            1 => LevelFilter::Debug,
            2 => LevelFilter::Trace,
            _ => LevelFilter::Info,
        })
        .format(|buf, record| {
            if record.level() != Level::Info
                || log_enabled!(Level::Trace)
                || log_enabled!(Level::Debug)
            {
                return writeln!(
                    buf,
                    "{}: {}",
                    record.level().to_string().to_lowercase(),
                    record.args()
                );
            }
            writeln!(buf, "{}", record.args())
        });

    cli.command().handler(logger)?;

    Ok(())
}
