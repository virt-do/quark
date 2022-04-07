# Quark

<img src="https://img.shields.io/github/workflow/status/virt-do/quark/quark%20build%20and%20unit%20tests?style=for-the-badge" />

## Build a quardle

Building a `quardle` with the container image bundled into the initramfs image :

```bash
quark build  --image <container_image_url> --offline --quardle <quardle_name>
```

Building a `quardle` with the container image to be pulled from within the guest:

```bash
quark build  --image <container_image_url> --quardle <quardle_name>
```

## Run a quardle

```bash
quark run --quardle <quardle> --output <output_file>
```
