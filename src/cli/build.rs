use clap::Args;

use super::{Error as QuarkError, Handler};

use flate2::{write::GzEncoder, Compression};
use git2::Repository;
use serde::{Deserialize, Serialize};
use std::fs::{copy, remove_dir_all, remove_file, File};
use std::io::{BufRead, BufReader, Error, ErrorKind};
use std::path::Path;
use std::process::{Command, Stdio};
use tar::Builder;

const BUNDLE_DIR: &str = "ctr-bundle/";
const CONFIG_FILE: &str = "quark.json";
const DEFAULT_CONTAINER_IMAGE_URL: &str = "https://dl-cdn.alpinelinux.org/alpine/v3.14/releases/x86_64/alpine-minirootfs-3.14.2-x86_64.tar.gz";
const INITRAMFS_NAME: &str = "initramfs.img";
const KAPS_PATH: &str = "kaps/target/x86_64-unknown-linux-musl/release/kaps";
const KERNEL_CMDLINE: &str = "console=ttyS0 i8042.nokbd reboot=k panic=1 pci=off";
const KERNEL_PATH: &str = "linux-cloud-hypervisor/arch/x86/boot/compressed/vmlinux.bin";
const ROOTFS_DIR: &str = "alpine-minirootfs/";

/// Builds a quardle, containing a VM rootfs, a linux kernel,
/// kaps binary and an optional container image bundle
#[derive(Debug, Args)]
#[clap(about)]
pub struct BuildCommand {
    /// The name of the generated quardle (the suffix `.qrk` will be added)
    #[clap(short, long)]
    quardle: String,

    /// The container image url to use. It should be an URL of a `.tar.gz` archive
    #[clap(short, long, default_value = DEFAULT_CONTAINER_IMAGE_URL)]
    image: String,

    /// Indicates if the container image should be bundled into the VM rootfs
    #[clap(short, long)]
    offline: bool,

    /// Overrides the default kernel command line
    #[clap(short, long, default_value = KERNEL_CMDLINE)]
    kernel_cmdline: String,
}

/// Describes the content of the `quark.json` config file
#[derive(Serialize, Deserialize, Debug)]
struct QuarkFile {
    /// The name of the generated quardle
    quardle: String,
    /// The kernel file name
    kernel: String,
    /// The initramfs image file name
    initramfs: String,
    /// The kernel command line
    kernel_cmdline: String,
    /// The container image url to use
    image: String,
    /// The kaps binary path
    kaps: String,
    /// States if the initramfs contains the container image
    offline: bool,
    /// If offline, states the name of the container image directory
    bundle: Option<String>,
}

/// Method that will be called when the command is executed.
impl Handler for BuildCommand {
    fn handler(&self) -> Result<(), QuarkError> {
        // Fetch & Build all prerequisites
        PreBuild::build_kaps("https://github.com/virt-do/kaps.git")?;
        PreBuild::build_kernel()?;
        if self.offline {
            PreBuild::build_bundle(&self.image)?;
        }
        PreBuild::build_rootfs(self.offline)?;

        // Create the JSON config file
        let config_file = QuarkFile {
            quardle: self.quardle.clone(),
            kernel: "vmlinux.bin".to_string(),
            initramfs: INITRAMFS_NAME.to_string(),
            image: self.image.clone(),
            kernel_cmdline: self.kernel_cmdline.clone(),
            kaps: "/opt/kaps".to_string(),
            offline: self.offline,
            bundle: if self.offline {
                Some(format!("/{}", BUNDLE_DIR))
            } else {
                None
            },
        };

        serde_json::to_writer_pretty(File::create(CONFIG_FILE)?, &config_file)
            .map_err(QuarkError::Serialize)?;
        // Create the archive
        println!("Creating the archive...");
        let quardle_name: &str = &format!("{}.qrk", self.quardle);
        let mut archive = Builder::new(GzEncoder::new(
            File::create(quardle_name)?,
            Compression::default(),
        ));
        archive.append_file(CONFIG_FILE, &mut File::open(CONFIG_FILE)?)?;
        archive.append_file("vmlinux.bin", &mut File::open(KERNEL_PATH)?)?;
        archive.append_file(INITRAMFS_NAME, &mut File::open(INITRAMFS_NAME)?)?;

        archive.finish()?;

        println!("{} has been created.", quardle_name);

        // Clean temporary files and directories (keeping kaps and kernel beceause of their size)
        remove_dir_all(ROOTFS_DIR)?;
        remove_dir_all(BUNDLE_DIR)?;
        remove_file(INITRAMFS_NAME)?;
        remove_file(CONFIG_FILE)?;
        Ok(())
    }
}

pub struct PreBuild {}
impl PreBuild {
    /// When kaps will have its own release on GitHub, the binary could be fetched here.
    /// Meanwhile, we are using git2 library to clone Kaps from GitHub, then cargo to build it from the sources.
    /// If the kaps binary exists, skipping this step.
    pub fn build_kaps(kaps_repository_url: &str) -> Result<(), QuarkError> {
        if Path::new(KAPS_PATH).exists() {
            println!("Kaps binary already exists, skipping.");
            return Ok(());
        }
        println!("Cloning the Kaps GitHub repository...");
        Repository::clone(kaps_repository_url, "kaps").map_err(QuarkError::Git)?;
        // Since there is a bug in the latests versions of kaps to build it with a musl target,
        // we need to checkout to a working version.
        Command::new("sh")
            .arg("-c")
            .arg("cd kaps && git checkout cdce0eb")
            .output()?;

        println!("Building kaps binary...");
        let stdout = Command::new("sh")
            .arg("-c")
            .arg("cd kaps && cargo build --release --target=x86_64-unknown-linux-musl")
            .stdout(Stdio::piped())
            .spawn()?
            .stdout
            .ok_or_else(|| Error::new(ErrorKind::Other, "Failed to execute cargo build"))?;
        let reader = BufReader::new(stdout);
        reader
            .lines()
            .filter_map(|line| line.ok())
            .for_each(|line| println!("{}", line));
        Ok(())
    }

    /// Building kernel from cloud-hypervisor/linux Git repository, with a *x86_64* config.
    /// If the `kernel` exists, skipping this step.
    pub fn build_kernel() -> Result<(), QuarkError> {
        if Path::new(KERNEL_PATH).exists() {
            println!("Kernel already exists, no need to re-build it, skipping.");
            return Ok(());
        }
        println!("Building kernel...");
        let stdout = Command::new("bash")
            .arg("-c")
            .arg("kernel/mkkernel.sh")
            .stdout(Stdio::piped())
            .spawn()?
            .stdout
            .ok_or_else(|| Error::new(ErrorKind::Other, "Failed to build kernel"))?;
        let reader = BufReader::new(stdout);
        reader
            .lines()
            .filter_map(|line| line.ok())
            .for_each(|line| println!("{}", line));
        Ok(())
    }

    /// Creates the container bundle, containing the image inside a rootfs folder
    /// and a runc spec (config.json).
    pub fn build_bundle(image_url: &str) -> Result<(), QuarkError> {
        if Path::new(BUNDLE_DIR).exists() {
            println!("Container bundle already exists, skipping.");
            return Ok(());
        }
        println!("Creating container bundle...");
        let stdout = Command::new("bash")
            .arg("-c")
            .arg(format!("scripts/mkbundle.sh {}", image_url))
            .stdout(Stdio::piped())
            .spawn()?
            .stdout
            .ok_or_else(|| Error::new(ErrorKind::Other, "Failed to build bundle"))?;
        let reader = BufReader::new(stdout);
        reader
            .lines()
            .filter_map(|line| line.ok())
            .for_each(|line| println!("{}", line));

        Ok(())
    }

    /// Creates the alpine-minirootfs folder, fetched from
    /// https://dl-cdn.alpinelinux.org/alpine/v3.14/releases/x86_64/alpine-minirootfs-3.14.2-x86_64.tar.gz,
    /// with an init file, and copy kaps binary (and container bundle if offline mode).
    /// If the folder exists, skipping this step.
    pub fn build_rootfs(offline: bool) -> Result<(), QuarkError> {
        if Path::new(INITRAMFS_NAME).exists() {
            println!("rootfs image already exists, skipping.");
            return Ok(());
        }
        BufReader::new(
            Command::new("bash")
                .arg("-c")
                .arg("scripts/mkrootfs.sh")
                .stdout(Stdio::piped())
                .spawn()?
                .stdout
                .ok_or_else(|| Error::new(ErrorKind::Other, "Failed to build rootfs"))?,
        )
        .lines()
        .filter_map(|line| line.ok())
        .for_each(|line| println!("{}", line));
        // Adding Kaps binary to the rootfs
        copy(KAPS_PATH, format!("{}opt/kaps", ROOTFS_DIR))?;

        if offline {
            // Adding the container bundle to the rootfs
            // (Cannot use copy function from std::fs because there is a lot of files to move)
            BufReader::new(
                Command::new("cp")
                    .arg("-r")
                    .arg(BUNDLE_DIR)
                    .arg(ROOTFS_DIR)
                    .stdout(Stdio::piped())
                    .spawn()?
                    .stdout
                    .ok_or_else(|| {
                        Error::new(ErrorKind::Other, "Failed to copy bundle into rootfs")
                    })?,
            )
            .lines()
            .filter_map(|line| line.ok())
            .for_each(|line| println!("{}", line));
        }

        println!("Creating initramfs image...");
        BufReader::new(
            Command::new("bash")
                .arg("-c")
                .arg(format!(
                    "cd {} && find . -print0 |
            cpio --null --create --owner root:root --format=newc |
            xz -9 --format=lzma  > ../{}",
                    ROOTFS_DIR, INITRAMFS_NAME
                ))
                .stdout(Stdio::piped())
                .spawn()?
                .stdout
                .ok_or_else(|| Error::new(ErrorKind::Other, "Failed to build initramfs image"))?,
        )
        .lines()
        .filter_map(|line| line.ok())
        .for_each(|line| println!("{}", line));

        Ok(())
    }
}
