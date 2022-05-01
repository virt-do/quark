use flate2::read::GzDecoder;
use log::info;
use std::{fs::File, io::Result, path::Path};
use tar::Archive;

/// Extract quardle archive to the output directory
pub fn extract_quardle(output: &str, quardle: &str) -> Result<()> {
    if !Path::new(output).exists() {
        let tar_gz = File::open(quardle)?;
        let tar = GzDecoder::new(tar_gz);
        let mut archive = Archive::new(tar);

        info!("Unpacking quardle...");
        archive.unpack(output)?;
        info!("Done");
    } else {
        info!("quardle already exists");
    }

    Ok(())
}
