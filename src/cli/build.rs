use clap::Args;

use super::{Handler, Result};
use serde::{Deserialize, Serialize};
use std::io::Write;

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonConfig {
    kernel: String,
    initrd: String,
    container_url: String,
}

// pub enum Result <Success, Error> {
//     Ok(Success),
//     Err(Error),
// }

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

    /// Overrides the default kernel image
    #[clap(
        short,
        long,
        default_value = "./build/lumper/kernel/linux-cloud-hypervisor"
    )]
    kernel: String,

    /// Overrides the default rootfs
    #[clap(short, long, default_value = "./build/lumper/rootfs/alpine-minirootfs")]
    rootfs: String,

    /// Path for kaps
    #[clap(
        short = 'K',
        long,
        default_value = "./build/kaps/target/x86_64-unknown-linux-musl/release/kaps"
    )]
    kaps: String,

    /// Overrides the default bundle
    #[clap(short, long)]
    bundle: String,
}

fn copy_dir(src: &str, workdir: &str) -> Result<()> {
    println!("Copying directory {} to {}", src, workdir);
    std::process::Command::new("cp")
        .arg("-r")
        .arg(src)
        .arg(workdir)
        .status()
        .unwrap();

    Ok(())
}

/// Method that will be called when the command is executed.
impl Handler for BuildCommand {
    fn handler(&self) -> Result<()> {
        // If offline is not set, display error message
        let workdir = "/tmp/quark/";

        if !self.offline {
            println!("Online mode is not supported yet.");
            return Ok(());
        }

        // If workdir exists, remove it
        if std::path::Path::new(workdir).exists() {
            println!("Removing existing workdir {}", workdir);
            std::fs::remove_dir_all(workdir).unwrap();
        }

        // Create the workdir
        println!("Creating workdir {}", workdir);
        std::fs::create_dir_all(&workdir)?;

        // If the kernel is a directory, copy it into the workdir
        if std::fs::metadata(&self.kernel).is_ok() {
            copy_dir(&self.kernel, workdir)?;
        } else {
            // If the kernel doesn't exist, display error message
            println!("Kernel doesn't exist.");
            return Ok(());
        }

        // If the rootfs is a directory, copy it into the workdir
        if std::fs::metadata(&self.rootfs).is_ok() {
            copy_dir(&self.rootfs, workdir)?;
        } else {
            // If the rootfs doesn't exist, display error message
            println!("Rootfs doesn't exist.");
            return Ok(());
        }

        // Get the rootfs filename
        let rootfs_path = std::path::Path::new(&self.rootfs);
        let rootfs_filename = rootfs_path.file_name().unwrap().to_str().unwrap();

        // If kaps exists, copy it into the workdir rootfs
        if std::fs::metadata(&self.kaps).is_ok() {
            println!(
                "Copying kaps to workdir rootfs {}{}",
                workdir, rootfs_filename
            );

            std::fs::copy(&self.kaps, &format!("{}/{}/kaps", workdir, rootfs_filename))?;
        } else {
            // If kaps doesn't exist, display error message
            println!("Kaps doesn't exist.");
            return Ok(());
        }

        // If bundle exists, copy  the directory into the workdir rootfs
        if std::fs::metadata(&self.bundle).is_ok() {
            copy_dir(&self.bundle, &format!("{}{}", workdir, rootfs_filename))?;
        } else if self.bundle.contains("http") {
            // If bundle is a url, download it
            /* TODO: implement */
            // display error message
            println!("Bundle is a url, not implemented yet.");
            return Ok(());
        } else {
            // If bundle doesn't exist, display error message
            println!("Bundle doesn't exist.");
            return Ok(());
        }

        // Configure rootfs
        println!("Configuring rootfs");

        // Change the init script to use the kaps bundle
        println!("Changing init script to use the kaps bundle");
        let init_script = format!("{}/{}/init", workdir, rootfs_filename);
        // Remove the old init script
        std::fs::remove_file(init_script.clone()).unwrap();

        // Write the new init script
        let mut init_script_file = std::fs::File::create(init_script.clone())?;
        println!("Writing new init script");
        let kaps_name = self.kaps.split('/').last().unwrap();
        let bundle_name = self.bundle.split('/').last().unwrap();
        let init_script_content = format!(
            "mount -t devtmpfs dev /dev\nmount -t proc proc /proc\nmount -t sysfs sys /sys\nip link set lo up\necho running kaps\n/{} run --bundle {}\n",
            kaps_name, bundle_name
        );
        write!(&mut init_script_file, "{}", init_script_content)?;

        // Make init script executable
        println!("Making init script executable");
        std::process::Command::new("chmod")
            .arg("+x")
            .arg(init_script)
            .status()
            .unwrap();

        // Create initramfs
        println!("Creating initramfs");
        let rootfs_workdir = format!("{}/{}", workdir, rootfs_filename);
        (subprocess::Exec::shell(format!("find {} -print0", rootfs_workdir))
            | subprocess::Exec::shell("cpio --null --create --owner root:root --format=newc")
            | subprocess::Exec::shell(format!("xz -9 --format=lzma  > {}initramfs.img", workdir)))
        .join()
        .unwrap();
        println!("Initramfs created");

        // Generate the quark.json file
        println!("Generating quark.json");
        let json_data = JsonConfig {
            kernel: self.kernel.clone().split('/').last().unwrap().to_string(),
            initrd: "initramfs.img".to_string(),
            container_url: bundle_name.to_string(),
        };
        let serialized = serde_json::to_string(&json_data).unwrap();
        // Write the quark.json file
        let quark_json = format!("{}quark.json", workdir);
        let mut quark_json_file = std::fs::File::create(quark_json.clone())?;
        write!(&mut quark_json_file, "{}", serialized)?;
        println!("Quark.json generated");

        // Create the quardle file
        let quark_name = if self.quardle.ends_with(".qrk") {
            self.quardle.clone()
        } else {
            format!("{}.qrk", self.quardle)
        };
        println!("Creating {}tar directory", workdir,);
        // Create the directory for the quardle file
        std::fs::create_dir_all(format!("{}tar", workdir))?;
        // Copy kernel directory into the quark directory
        copy_dir(&self.kernel, &format!("{}tar", workdir))?;
        // Copy initramfs into the quark directory
        println!("Copying initramfs to tar directory");
        std::fs::copy(
            &format!("{}initramfs.img", workdir),
            &format!("{}tar/initramfs.img", workdir),
        )?;
        // Copy quark.json into the quark directory
        println!("Copying quark.json to tar directory");
        std::fs::copy(&quark_json, &format!("{}tar/quark.json", workdir))?;

        // Create the quardle file
        println!("Creating {} file", quark_name);

        // Create the tar file
        std::process::Command::new("tar")
            .arg("-czf")
            .arg(quark_name.clone())
            .arg(format!("{}tar", workdir))
            .status()
            .unwrap();

        println!("{} file created", quark_name);

        Ok(())
    }
}
