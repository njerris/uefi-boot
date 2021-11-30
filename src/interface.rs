//! Definitions for the uefi-boot application interface

/// The magic number.
pub const MAGIC: u64 = 0xfedcba9876543210;

/// Boot information data structure.
/// 
/// This structure provides information necessary for the loaded application
/// to take control of the system.
/// 
/// The loaded application must understand that all provided pointers are
/// strictly physical addresses. If the application unmaps the system identity
/// mapping of all physical memory, it must adjust those pointers accordingly,
/// or not use them at all.
pub struct BootInfo {
    /// Pointer to the EFI memory map.
    pub efi_mmap_start: usize,
    /// Length of the EFI memory map.
    pub efi_mmap_length: usize,
    /// The size of each EFI descriptor entry.
    pub efi_mmap_desc_size: usize,

    /// The start of the ramdisk in memory.
    pub ramdisk_start: usize,
    /// The length of the ramdisk in bytes.
    pub ramdisk_length: usize,

    /// A pointer to the EFI system table.
    pub efi_system_table: usize,
    /// A pointer to the active graphics output protocol mode structure.
    pub efi_gop_modes: Option<usize>,
}