use std::{error::Error, fs::File, io::BufReader, path::Path};

use serde::Deserialize;

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct QuarkConfig {
    pub quardle: String,
    pub kernel: String,
    pub initramfs: String,
    pub kernel_cmdline: String,
    pub image: String,
    pub kaps: String,
    pub offline: bool,
    pub bundle: String,
}

pub fn read_config_from_file<P: AsRef<Path>>(path: P) -> Result<QuarkConfig, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    // Read the JSON contents of the file as an instance of `QuarkConfig`.
    let config = serde_json::from_reader(reader)?;

    Ok(config)
}
