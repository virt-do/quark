use crate::quardle::Quardle;
use crate::{Handler, Result};
use clap::Args;

/// CLI related errors
#[derive(Debug)]
pub enum Error {
}

/// Arguments for our `BuildCommand`.
///
/// These arguments are parsed by `clap` and an instance of `BuildCommand` containing
/// arguments is provided.
///
/// Example :
///
/// `quark build --image <container_image_url> --quardle <quardle_name>`
///
/// The `handler` method provided below will be executed.
#[derive(Debug, Args)]
pub struct BuildCommand {
  /// The url of the container image
  #[clap(short, long)]
  image: String,
  /// The name of the quardle to create
  #[clap(short, long)]
  quardle: String,

  /// The name of the quardle to create
  #[clap(short, long)]
  offline: bool
}

impl Handler for BuildCommand {
  fn handler(&self) -> Result<()> {
    Quardle::new().unwrap();

    Ok(())
  }
}
