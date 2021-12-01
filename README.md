# uefi-boot
UEFI app to load 64-bit ELF executables

## Objectives
`uefi-boot` is intended to be a minimal, easy-to-use bootloader for UEFI systems that loads 64-bit ELF executables and ramdisks. For now, only the x86_64 architecture is supported, but aarch64 and risc-v are possible future targets. It is designed to support loading executables into the higher-half of memory in a full 64-bit environment with paging enabled. The hope is that this will reduce the work required to get started with OS kernel development.

Previously, if one desired to create a 64-bit higher-half kernel, there were two main options:
1. Use a bootloader such as GRUB2, which requires complex bootstrap assembly code to transition from a 32-bit physical address environment to a 64-bit environment with paging enabled.
2. Create the kernel as a UEFI application, which requires the kernel to re-map itself into the higher-half and the use of the PE32+ format and the Microsoft x64 calling convention.

## Interface
`uefi-boot` provides a magic number and a boot information data structure to the kernel entry function. See `src/lib.rs` for detailed information.

## Dependencies
You must have the Rust nightly toolchain installed: `rustup toolchain install nightly`. Additionally, you need `cargo-xbuild` for cross-compilation: `cargo install cargo-xbuild`.
All other dependencies are managed by `cargo`.

## Build Instructions
Run `build.sh` to build and `clean.sh` to clean the directory.