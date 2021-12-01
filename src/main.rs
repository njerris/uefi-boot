//! UEFI app to load 64-bit ELF executables

#![no_main]
#![no_std]
#![feature(asm)]
#![feature(lang_items)]
#![feature(proc_macro_hygiene)]

#[macro_use]
mod env;

#[cfg(target_arch = "x86_64")]
#[path = "arch/x86_64.rs"]
mod arch;

mod graphics;
mod interface;
mod loader;

use interface::BootInfo;
use r_efi::efi;
use utf16_lit::utf16;

// Static pointers to the UEFI system table and filesystem root.
static mut ST: *const efi::SystemTable = 0 as *const _;
static mut ROOT: *mut efi::protocols::file::Protocol = 0 as *mut _;

// Hard-coded paths to kernel and ramdisk.
const KERNEL_PATH: &[u16] = &utf16!("uefi-boot\\kernel.elf64\0");
const RAMDISK_PATH: &[u16] = &utf16!("uefi-boot\\init.rd\0");

// Entry point, called by the EFI.
#[export_name = "efi_main"]
pub extern "C" fn main(image_handle: efi::Handle, st: *mut efi::SystemTable) {
    // Set the system table pointer so that printing will work.
    unsafe {
        ST = st;
    }

    println!("uefi-boot running...");

    env::init_fs(image_handle);

    // If either the kernel or ramdisk is not present, panic.
    let kfile =
        env::open_file(KERNEL_PATH.as_ptr() as *mut _).expect("failed to open kernel executable");
    let rdfile =
        env::open_file(RAMDISK_PATH.as_ptr() as *mut _).expect("failed to open ramdisk file");

    arch::prepare_root_pt();

    // Load the kernel and ramdisk into memory.
    let entry_fn_ptr = loader::load_kernel(kfile);
    let (rd_start, rd_length) = loader::load_ramdisk(rdfile);

    // Create the boot information structure.
    let info_buffer = env::allocate_pool(core::mem::size_of::<BootInfo>())
        .expect("failed to allocate buffer for the boot information structure");
    let info = unsafe { &mut *(info_buffer as *mut BootInfo) };
    info.ramdisk_start = rd_start;
    info.ramdisk_length = rd_length;
    info.efi_system_table = st as usize;
    info.efi_gop_modes = graphics::get_mode();

    println!("preparing kernel handoff...");

    // Get the memory map.
    let ((mmap, mmap_length, desc_size), mmap_key) = get_memory_map();
    info.efi_mmap_start = mmap;
    info.efi_mmap_length = mmap_length;
    info.efi_mmap_desc_size = desc_size;

    // Exit boot services.
    let status = unsafe { ((*(*ST).boot_services).exit_boot_services)(image_handle, mmap_key) };
    if status.is_error() {
        panic!("failed to exit UEFI boot services");
    }

    // Use sysv64 calling convention on x86_64.
    #[cfg(target_arch = "x86_64")]
    let entry: extern "sysv64" fn(magic: u64, info_ptr: usize);

    // Call the kernel's entry function.
    unsafe { 
        entry = core::mem::transmute(entry_fn_ptr);
        entry(interface::MAGIC, info_buffer);
    }

    // Kernel should never return.
    loop {}
}

// Get tuple (memory map pointer, memory map size, descriptor entry size, memory map key).
pub fn get_memory_map() -> ((usize, usize, usize), usize) {
    // Call boot_services.get_memory_map() with a buffer of size 0.
    // mmap_size will then hold the required size of the buffer.
    let mut mmap_size = 0usize;
    let mut mmap_key = 0usize;
    let mut descriptor_size = 0usize;
    let mut descriptor_version = 0u32;
    let status = unsafe {
        ((*(*ST).boot_services).get_memory_map)(
            &mut mmap_size as *mut usize,
            0 as *mut efi::MemoryDescriptor,
            &mut mmap_key as *mut usize,
            &mut descriptor_size as *mut usize,
            &mut descriptor_version as *mut u32,
        )
    };
    if !status.is_error() {
        panic!("get_memory_map pass 1 succeeded but should fail");
    }

    // Retry with a buffer of the correct size (plus a buffer if the allocation alters the map).
    let mmap_buffer =
        env::allocate_pool(mmap_size + 128).expect("failed to allocate buffer for memory map");
    let status = unsafe {
        ((*(*ST).boot_services).get_memory_map)(
            &mut mmap_size as *mut usize,
            mmap_buffer as *mut efi::MemoryDescriptor,
            &mut mmap_key as *mut usize,
            &mut descriptor_size as *mut usize,
            &mut descriptor_version as *mut u32,
        )
    };
    if status.is_error() {
        panic!("failed to get UEFI memory map");
    }
    if descriptor_version != efi::MEMORY_DESCRIPTOR_VERSION {
        panic!("incompatible UEFI memory map descriptor version");
    }

    ((mmap_buffer, mmap_size, descriptor_size), mmap_key)
}

// The panic handler simply prints a message and stalls.
#[panic_handler]
fn panic_handler(info: &core::panic::PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
