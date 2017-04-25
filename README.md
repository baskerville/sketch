# Description

This is a drawing program that is meant to be run on a *Kobo* e-reader.

It has only been tested on the *Glo HD* and the *Aura ONE*.

# Installation

First install [fmon](https://github.com/baskerville/fmon).

And then issue: `unzip sketch.zip -d SD_ROOT`.

# Usage

Use your fingers to draw.

A short press/release of the power button will:

- Save and clear the canvas if it isn't empty.
- Quit if it's empty.

A long press/release (hold more than 2 seconds) of the power button will inverse the displayed colors.

# Configuration

If the touch feedback doesn't match the position of your fingers, add the following:
```
export SKETCH_UNSWAP_XY=1
export SKETCH_UNMIRROR_X=1
```
in `sketch.sh` after `export PRODUCT…`.

# Building

The OS used on the *Kobo* devices is *Linaro 2011.07*.

In order to build for this OS / architecture you can, for example, install *Ubuntu LTS 12.04* (the *GLIBC* version must be old enough) in a VM and install the following package: `gcc-4.6-arm-linux-gnueabihf`.

Install the appropriate target:
```sh
rustup target add arm-unknown-linux-gnueabihf
```

Append this:
```toml
[target.arm-unknown-linux-gnueabihf]
linker = "arm-linux-gnueabihf-gcc"
```
to `~/.cargo/config`.

The binary can then be generated with:
```sh
cargo rustc --release --target=arm-unknown-linux-gnueabihf -- -C target-feature=+v7,+vfp3,+a9,+neon
```

You can tell what features are supported by your device from the output of `cat /proc/cpuinfo`.

