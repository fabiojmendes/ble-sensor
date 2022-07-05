# Build

Cross compiling for `arm-unknown-linux-gnueabi` (for raspberry pi zero w) on Debian 11

```
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
