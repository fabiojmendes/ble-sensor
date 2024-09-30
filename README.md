<!-- vim: set tw=80: -->

# Tempsys Scan

This repository contains the code for the scanning component of Tempsys. It will
read the advertising events emitted by the
[tempsys-firmware](https://github.com/fabiojmendes/tempsys-firmware) and publish
it to a mqtt broker.

## Configuration

A toml configuration file is necessary is necessary to provide the location for
the mqtt broker and also friendly names for each tempsys thermometer.

Here's a sample of what the configuration file should look like:

<https://github.com/fabiojmendes/tempsys-scan/blob/master/conf/config.toml#L1-L11>

## WIP

-

## Build

Cross compiling for `arm-unknown-linux-gnueabi` (for raspberry pi zero w) on
Debian 11

```shell
# Assuming the rust toolchain is installed using rustup:
# Add the target architecture
rustup target add arm-unknown-linux-gnueabi

# Add arm architecture for dependencies
dpkg --add-architecture armel

# Install dependencies
apt update && apt install -y build-essential gcc-arm-linux-gnueabi libdbus-1-dev:armel

# Set the correct sysroot and config path for pkg-config
export PKG_CONFIG_SYSROOT_DIR=/usr/lib/arm-linux-gnueabi
export PKG_CONFIG_PATH=/usr/lib/arm-linux-gnueabi/pkgconfig
```
