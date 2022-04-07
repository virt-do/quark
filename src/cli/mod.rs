mod build;
mod run;

use crate::cli::build::BuildCommand;
use crate::cli::run::RunCommand;
use clap::{Parser, Subcommand};

#[derive(Debug)]
pub enum Error {
    OpenFile(std::io::Error),
    Serialize(serde_json::Error),
    Git(git2::Error),
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::OpenFile(error)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait Handler {
    fn handler(&self) -> Result<()>;
}

/// Create a cli for quark
#[derive(Parser, Debug)]
#[clap(version, author)]
pub struct Cli {
    #[clap(subcommand)]
    pub(crate) command: Command,
}

impl Cli {
    /// Return the command used in the cli.
    pub fn command(self) -> Box<dyn Handler> {
        match self.command {
            Command::Run(cmd) => Box::new(cmd),
            Command::Build(cmd) => Box::new(cmd),
        }
    }
}

/// The enumeration of our commands.
///
/// Each of our commands should be listed in this enumeration with the following format :
/// CommandName(CommandHandler)
///
/// Example:
///
/// You want to add the `list` command:
///
/// List(ListCommand)
#[derive(Subcommand, Debug)]
pub enum Command {
    /// Run a quardle
    Run(RunCommand),
    /// Build a quardle
    Build(BuildCommand),
}
