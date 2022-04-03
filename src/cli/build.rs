use clap::Args;

use super::{Handler, Result};

/// Arguments for `BuildCommand`
///
/// Usage :
/// `quark build --image <IMAGE>`
#[derive(Debug, Args)]
pub struct BuildCommand {
    /// The name of the generated quardle, with the suffix `.qrk`
    #[clap(short, long)]
    quardle: String,

    /// Indicates wether or not the container image is bundled into the initramfs image
    #[clap(short, long)]
    offline: bool,

    /// Overrides the default kernel command line
    #[clap(short, long)]
    kernel_cmd: Option<String>,
}

/// Method that will be called when the command is executed.
impl Handler for BuildCommand {
    fn handler(&self) -> Result<()> {
        Ok(())
    }
}
