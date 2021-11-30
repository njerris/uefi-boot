//! # uefi-boot Application Interface
//! 
//! uefi-boot supplies a 64-bit magic number and a pointer to a boot
//! information structure when calling the entry function of the loaded
//! application. The entry function should have the following signature:
//! ```rust
//! extern "sysv64" fn(magic: u64, info_addr: usize);
//! ```
//! 
//! The entry function itself should validate the magic number before accessing
//! the boot information structure, in order to verify that it was called by
//! uefi-boot.
//! 
//! Note that this crate provides no functions to access the boot information
//! structure pointed to `info_addr`. This is deliberate - the structure lives
//! in identity-mapped physical memory and is only valid so long as the app
//! hasn't re-mapped it elsewhere. By forcing the user to explicitly convert
//! from a usize to a pointer or a reference, I hope to prevent such an issue
//! from occurring.

#![no_std]

mod interface;

use self::interface::MAGIC as MAGIC;
use self::interface::BootInfo as BootInfo;