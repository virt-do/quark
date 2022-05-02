use std::fs::{File, create_dir_all, remove_dir_all};
use serde::{Deserialize, Serialize};
use serde_json;
use std::process::Command;
use log::{info, warn};

/// Constances declarations
const QUARK_BUILD_DIR: &str = "/tmp/quark/builds/";
const QUARK_CONFIG_DIR: &str = "/opt/quark/";

const QUARDLE_KERNEL: &str = "vmlinux.bin";
const QUARDLE_KERNEL_CMDLINE: &str = "/proc/cmdline";
const QUARDLE_INITRD: &str = "initramfs.img";

/// Containers related errors
#[derive(Debug, thiserror::Error)]
pub enum Error {}

/// A common result type for our module.
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Serialize, Deserialize)]
struct QuardleConfig {
    name: String,
    kernel: String,
    initrd: String,
    cmdline: String,
    container_image_url: String,
}

/// The `Container` struct provides a simple way to
/// create and run a container on the host.
#[derive(Default, Debug, Clone)]
pub struct Quardle {
    name: String, 
    container_image_url: String, 
    offline: bool
}

impl Quardle {

    /// Build a quardle from instance variables
    /// If delete is true, delete the quardle temporary files after building
    pub fn build(&self, delete_after: Option<bool>) -> Result<()> {
        // creating working directory
        create_dir_all(self.clone().get_work_dir()).unwrap();

        self
            .setup()
            .add_config_file()
            .make_archive()?;
        
        // Deleting temporary files used to build the quardle
        if delete_after.unwrap_or(false) {
            self.delete().unwrap();
        }

        Ok(())
    }

    /// Instanciate a new quardle
    pub fn new(name: String, container_image_url: String, offline: bool) -> Option<Self> {
        Some(Quardle {name, container_image_url, offline})
    }

    /// Delete all temporary files used to create quardle are created at /tmp/quark/builds/<quardle_name>/
    pub fn delete(&self) -> Result<()> {
        if !std::path::Path::new(format!("{}", self.clone().get_work_dir()).as_str()).exists() {
            remove_dir_all(self.clone().get_work_dir()).unwrap();
        }
        Ok(())
    }

    /// Return the path to the quardle working directory
    /// /tmp/quark/builds/<quardle_name>/
    /// Used to create temporary files used to build quardle
    fn get_work_dir(self) -> String {
        format!("{}{}/", QUARK_BUILD_DIR, self.name)
    }

    /// Setup default quark configuration 
    /// Create a `quark` directory in /opt
    fn setup(&self) -> &Quardle {
        // Create the quark configuration directory
        if !std::path::Path::new(&format!("{}", QUARK_CONFIG_DIR)).exists() {
            warn!("Quark configuration directory does not exist, creating it !");
            Command::new("mkdir")
                .arg("/opt/quark")
                .output()
                .expect("failed to setup quark");
        }

        self
    }

    /// Generate the config file of the quardle
    /// The config file is a JSON file containing a QuardleConfig struct
    fn add_config_file(&self) -> &Quardle {
      
        let config = QuardleConfig {
            name: self.name.clone(),
            kernel: QUARDLE_KERNEL.to_string(),
            initrd: QUARDLE_INITRD.to_string(),
            cmdline: QUARDLE_KERNEL_CMDLINE.to_string(),
            container_image_url: self.container_image_url.clone()
        };

        let config_json = serde_json::to_string(&config).unwrap();
        let mut file = File::create(format!("{}quark.json",self.clone().get_work_dir())).unwrap();
        
        use std::io::Write;
        file.write_all(config_json.as_bytes()).unwrap();

        self
    }

    /// Create compressed archive from quardle files and append it to /out/<quardle_name>.qrk
    fn make_archive(&self) -> Result<()> {
        info!("Packaging quardle.");
        Command::new("tar")
            .arg("-zcvf")
            .arg(format!("{}",format!("out/{}.qrk",self.name)))
            .arg(format!("{}",self.clone().get_work_dir()))
            .output()
            .expect("failed to create archive");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quardle_new() {
        let quardle = Quardle::new("test1".to_string(), "container1".to_string(), false);
        assert_eq!(quardle.as_ref().unwrap().name, "test1");
        assert_eq!(quardle.as_ref().unwrap().container_image_url, "container1");
    }

    #[test]
    fn quardle_get_work_dir() {
        let quardle = Quardle::new("this-should-be-a-directory".to_string(), "my-container".to_string(), false);
        assert_eq!(quardle.unwrap().get_work_dir(), "/tmp/quark/builds/this-should-be-a-directory/");
    }
}