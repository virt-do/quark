# Quark

<img src="https://img.shields.io/github/workflow/status/virt-do/quark/quark%20build%20and%20unit%20tests?style=for-the-badge" />

## Getting started

### Building phase 

To run as an example; you can do this :

```sh
git clone git@github.com:virt-do/quark.git
cd quark
./hack/mk_all.sh
cargo build
sudo ./target/debug/quark -o -q hello.qrk
```
#### Test building phase without quark run

If we have a file like `hello.qrk`, we can test it like this

```sh
tar xzf hello.qrk
sudo ./build/lumper/target/debug/lumper --kernel tmp/quark/tar/linux-cloud-hypervisor/arch/x86/boot/compressed/vmlinux.bin -i tmp/quark/tar/initramfs.img 
```


## Troubleshooting

### Kaps, build target x86_64-unknown-linux-musl

#### error: failed to run custom build command for `openssl-sys v0.9.72`

If this happened during `./hack/mk_all.sh`, you need to go in `build/kaps/Cargo.toml` and add this in dependencies

```cargo
openssl = { version = "0.10", features = ["vendored"] }
```

Relaunch `./hack/mk_all.sh` after this
