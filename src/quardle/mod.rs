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
            .add_kernel()
            .add_initramfs()
            .add_config_file()
            .clean_quardle_build_dir()
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

        // Install kaps sources
        if !std::path::Path::new(&format!("{}kaps", QUARK_CONFIG_DIR)).exists() {
            info!("Kaps not found, installing it !");
            Command::new("git")
                .args(["clone", "https://github.com/virt-do/kaps.git"]) // Using https protocol because it seems not supporting ssh
                .current_dir(format!("{}", QUARK_CONFIG_DIR))
                .output()
                .expect("failed to fetch kaps");
        }

        // Install a default kernel configuration file
        if !std::path::Path::new(&format!("{}linux-config-x86_64", QUARK_CONFIG_DIR)).exists() {
            warn!("Kernel config file not found, installing it !");
            Command::new("curl")
                .arg("https://raw.githubusercontent.com/virt-do/lab/main/do-vmm/kernel/linux-config-x86_64")
                .arg("-O") 
                .current_dir(format!("{}", QUARK_CONFIG_DIR))
                .output()
                .expect("failed to fetch kernel config file");
        }

        // Install a script to build kernel
        if !std::path::Path::new(&format!("{}mkkernel.sh", QUARK_CONFIG_DIR)).exists() {
            warn!("Kernel build script not found, installing it !");
            Command::new("curl")
                .arg("https://raw.githubusercontent.com/virt-do/lab/main/do-vmm/kernel/mkkernel.sh")
                .arg("-O") 
                .current_dir(format!("{}", QUARK_CONFIG_DIR))
                .output()
                .expect("failed to fetch kernel build script");

            Command::new("chmod")
                .arg("+x")
                .arg(format!("{}mkkernel.sh", QUARK_CONFIG_DIR))
                .current_dir(format!("{}", QUARK_CONFIG_DIR))
                .output()
                .expect("failed to make kernel build script executable");
        }

        // Building kernel binary
        if !std::path::Path::new(&format!("{}linux-cloud-hypervisor", QUARK_CONFIG_DIR)).exists() {
            warn!("Kernel not builded, building it !");
            Command::new(format!("{}mkkernel.sh", QUARK_CONFIG_DIR))
                .current_dir(format!("{}", QUARK_CONFIG_DIR))
                .output()   
                .expect("failed to build kernel");
        }

        // Install a script to build initramfs
        if !std::path::Path::new(&format!("{}alpine-minirootfs", QUARK_CONFIG_DIR)).exists() {
            warn!("Rootfs not builded, building it !");
            Command::new("curl")
                .arg("https://dl-cdn.alpinelinux.org/alpine/v3.14/releases/x86_64/alpine-minirootfs-3.14.2-x86_64.tar.gz")
                .arg("-O") 
                .current_dir(format!("{}", QUARK_CONFIG_DIR))
                .output()
                .expect("failed to download rootfs archive");
            Command::new("mkdir")
                .arg("alpine-minirootfs")
                .current_dir(format!("{}", QUARK_CONFIG_DIR))
                .output()
                .expect("failed to download initramfs build script");
            Command::new("tar")
                .arg("-xzf")
                .arg("alpine-minirootfs-3.14.2-x86_64.tar.gz")
                .arg("-C")
                .arg("alpine-minirootfs")
                .current_dir(format!("{}", QUARK_CONFIG_DIR))
                .output()
                .expect("failed to extract rootfs archive");
            
            // Adding kaps binary to the rootfs
            self.add_kaps_to_rootfs();
        }
        
        if !std::path::Path::new(&format!("{}mkinitramfs.sh", QUARK_CONFIG_DIR)).exists() {
            warn!("InitramFS build script not found, installing it !");
            Command::new("curl")
                .arg("https://raw.githubusercontent.com/virt-do/quark/main/tools/mkinitramfs.sh")
                .arg("-O") 
                .current_dir(format!("{}", QUARK_CONFIG_DIR))
                .output()
                .expect("failed to download initramfs build script");
            Command::new("chmod")
                .arg("+x")
                .arg(format!("{}mkinitramfs.sh", QUARK_CONFIG_DIR))
                .current_dir(format!("{}", QUARK_CONFIG_DIR))
                .output()
                .expect("failed to make initramfs build script executable");
        }

        // Install a script to build kaps bundle
        if !std::path::Path::new(&format!("{}mkbundle.sh", QUARK_CONFIG_DIR)).exists() {
            warn!("Kaps bundle build script not found, installing it !");
            Command::new("curl")
                .arg("https://raw.githubusercontent.com/virt-do/lab/main/do-vmm/rootfs/mkbundle.sh")
                .arg("-O") 
                .current_dir(format!("{}", QUARK_CONFIG_DIR))
                .output()
                .expect("failed to fetch kaps bundle build script");
            Command::new("chmod")
                .arg("+x")
                .arg(format!("{}mkbundle.sh", QUARK_CONFIG_DIR))
                .current_dir(format!("{}", QUARK_CONFIG_DIR))
                .output()
                .expect("failed to make kernel build script executable");
        }
        self
    }

    /// Append kernel configuration
    /// Fetch automated script if isn't already installed, and use some bash script to build it
    fn add_kernel(&self) -> &Quardle {
        info!("Installing kernel binary !");
        Command::new("cp")
            .arg(format!("{}linux-cloud-hypervisor/arch/x86/boot/compressed/vmlinux.bin", QUARK_CONFIG_DIR))
            .arg(format!("{}", self.clone().get_work_dir()))
            .spawn()
            .expect("failed to copy kernel");
        self
    }

    /// Append basic rootfs
    /// Fetch automated script if isn't already installed, and use some bash script to build it
    fn add_initramfs(&self) -> &Quardle {
        info!("Installing initRamFS image to quardle");
        Command::new("cp")
            .arg("-r")
            .arg(format!("{}alpine-minirootfs", QUARK_CONFIG_DIR))
            .arg(format!("{}", self.clone().get_work_dir()))
            .spawn()
            .expect("failed to copy rootfs");

        // If offline mode is active, we need to build kaps bundle image directly in the quardle.
        if self.offline {
            info!("Offline mode, adding kaps bundle to the quardle.");
            Command::new(format!("{}mkbundle.sh", QUARK_CONFIG_DIR))
            .arg(format!("{}/alpine-minirootfs/ctr-bundle", self.clone().get_work_dir()))
            .current_dir(format!("{}", QUARK_CONFIG_DIR))
            .output()   
            .expect("failed to build initramfs");
        }
            
        // add init file to it
        info!("InitramFS not builded, building it !");
        Command::new(format!("{}mkinitramfs.sh", QUARK_CONFIG_DIR))
            .arg(format!("{}/alpine-minirootfs", self.clone().get_work_dir()))
            .current_dir(format!("{}", QUARK_CONFIG_DIR))
            .output()   
            .expect("failed to build initramfs");
     
        self
    }

    fn clean_quardle_build_dir(&self) -> &Quardle {
        info!("Cleaning quardle build directory !");
        Command::new("rm")
            .arg("-rdf")
            .arg("alpine-minirootfs")
            .arg("mkbundle.sh")
            .arg("mkinitramfs.sh")
            .current_dir(format!("{}", self.clone().get_work_dir()))
            .output()
            .expect("failed to clean quardle build directory");
        self
    }

    /// Append kaps binary to rootfs image
    /// Fetch kaps source code if isn't already installed, build it from source and copy it to the working directory
    fn add_kaps_to_rootfs(&self) -> &Quardle {
        info!("Installing kaps to quardle");
        Command::new("cargo")
            .current_dir(format!("{}kaps", QUARK_CONFIG_DIR))
            .arg("build")
            .arg("--release")
            // .arg("--out-dir") //TODO: outdir is only available on nightly for now, should be used later
            // .arg(format!("{}/rootfs/usr/bin/kaps",self.clone().get_work_dir()))
                .output()
                .expect("failed to build kaps");

        Command::new("cp")
            .arg(format!("{}kaps/target/release/kaps", QUARK_CONFIG_DIR))
            .arg(format!("{}/alpine-minirootfs/usr/bin/kaps",self.clone().get_work_dir()))
            .output()
            .expect("failed to copy kaps");

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

    #[test]
    fn quardle_setup() {
        let quardle = Quardle::new("test2".to_string(), "container2".to_string(), false);
        quardle.as_ref().unwrap().setup();
        assert_eq!(quardle.as_ref().unwrap().name, "test2");
        assert_eq!(quardle.as_ref().unwrap().container_image_url, "container2");
    }

    #[test]
    fn quardle_add_kernel() {
        let quardle = Quardle::new("test3".to_string(), "container3".to_string(), false);
        quardle.as_ref().unwrap().add_kernel();
        assert_eq!(quardle.as_ref().unwrap().name, "test3");
        assert_eq!(quardle.as_ref().unwrap().container_image_url, "container3");
    }

    #[test]
    fn quardle_add_initramfs() {
        let quardle = Quardle::new("test4".to_string(), "container4".to_string(), false);
        quardle.as_ref().unwrap().add_initramfs();
        assert_eq!(quardle.as_ref().unwrap().name, "test4");
        assert_eq!(quardle.as_ref().unwrap().container_image_url, "container4");
    }

    #[test]
    fn quardle_add_kaps_to_rootfs() {
        let quardle = Quardle::new("test6".to_string(), "container6".to_string(), false);
        quardle.as_ref().unwrap().add_initramfs().add_kaps_to_rootfs();
        assert_eq!(quardle.as_ref().unwrap().name, "test6");
        assert_eq!(quardle.as_ref().unwrap().container_image_url, "container6");
    }

    #[test]
    #[should_panic]
    fn quardle_clean_quardle_build_dir() {
        let quardle = Quardle::new("test5".to_string(), "container5".to_string(), false);
        quardle.as_ref().unwrap().clean_quardle_build_dir();
        assert_eq!(quardle.as_ref().unwrap().name, "test5");
        assert_eq!(quardle.as_ref().unwrap().container_image_url, "container5");
    }
}