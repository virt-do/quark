# Quark

**quark** is a CLI tool for building and running images for lumper to boot.

<img src="https://img.shields.io/github/workflow/status/virt-do/quark/quark%20build%20and%20unit%20tests?style=for-the-badge" />

## How to use

Building a quardle with the container image bundled into the initramfs image:  
`$ quark build --image <container_image_url> --offline --quardle <quardle_name>`

Building a quardle with the container image to be pulled from within the guest:  
`$ quark build --image <container_image_url> --quardle <quardle_name>`

This commands will create a quardle with the name `<quardle_name>.qrk`
