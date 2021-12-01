#!/bin/sh
# uefi-boot build script

# Determine which architecture to build, default to x86_64
if [[ "$*" == "--arch=x86_64" ]]
then
    echo "target architecture set to x86_64"
    arch="x86_64"
else
    echo "target architecture not specified, defaulting to x86_64"
    arch="x86_64"
fi

# Determine if this is a debug or release build
if [[ "$*" == "--release" ]]
then
    echo "NOTE: release build"
    release=true
else
echo "NOTE: debug build, use --release for a release build"
    release=false
fi

# Determine target triple
if [[ "$arch" == "x86_64" ]]
then
    target="x86_64-unknown-uefi"
fi

# Build
echo "building uefi-boot"
cargo xbuild --target $target $1

# Copy binary to root
echo "copying binary"
if [[ $release == true ]]
then
    cp target/$target/release/uefi-boot.efi uefi-boot.efi
else
    cp target/$target/debug/uefi-boot.efi uefi-boot.efi
fi

echo "build complete"