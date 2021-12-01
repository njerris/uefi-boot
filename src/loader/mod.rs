// Loaders for kernels and ramdisks

mod elf64;

use crate::{arch, env, ST};
use elf64::{program::PHType, Elf64, ElfAbi, ElfType};
use r_efi::efi::protocols::file;

// Load the kernel into memory from a file, return the entry point.
pub fn load_kernel(kfile: *mut file::Protocol) -> usize {
    // Get the length of the kernel file.
    let info_buffer = env::allocate_pool(256).expect("failed to allocate file info buffer");
    let mut finfo_guid = file::INFO_ID;
    let mut size = 256; // whole page available for buffer
    let _ = unsafe {
        ((*kfile).get_info)(
            kfile,
            &mut finfo_guid,
            &mut size,
            info_buffer as *mut core::ffi::c_void,
        )
    };
    let kfile_len = unsafe { (*(info_buffer as *const file::Info)).file_size as usize };
    env::free_pool(info_buffer);

    // Load the kernel file contents into memory.
    assert_ne!(kfile_len, 0, "kernel file length must not be zero");
    let n = kfile_len / arch::PAGE_SIZE + 1;
    let kfile_start_page = env::allocate_pages(n).expect("failed to allocate kernel file pages");
    let _ = unsafe { ((*kfile).set_position)(kfile, 0) };
    let status = unsafe {
        ((*kfile).read)(
            kfile,
            &mut (kfile_len as usize),
            kfile_start_page as *mut core::ffi::c_void,
        )
    };
    if status.is_error() {
        panic!("failed to read contents of kernel file");
    }

    // Try to read the kernel file as an ELF-64 executable.
    let slice = unsafe { core::slice::from_raw_parts(kfile_start_page as *const u8, kfile_len) };
    let elf = Elf64::from_slice(slice).expect("unable to parse kernel file as ELF-64");

    // Check some ELF header fields to see if efiloader can load it.
    assert!(
        elf.is_valid_locally(),
        "the kernel ELF is not for this machine"
    );
    assert_eq!(
        elf.abi(),
        ElfAbi::None,
        "the kernel ELF requires ABI extensions to load"
    );
    assert_eq!(elf.abi_version(), 0, "the kernel ELF ABI version is not 0");
    assert_eq!(
        elf.file_type(),
        ElfType::Executable,
        "the kernel ELF is not executable"
    );

    for segment in elf.program_headers().expect("the kernel ELF is corrupt") {
        // Map only loadable segments.
        if segment.type_() == PHType::Load {
            assert!(
                arch::check_page_alignment(segment.offset as usize),
                "ELF segments must be 4k aligned"
            );
            assert!(
                arch::check_page_alignment(segment.vaddr as usize),
                "ELF segments must be 4k aligned"
            );
            assert!(elf.contains(segment), "the kernel ELF is corrupt");

            // Calculate how many pages come from the file vs. must be allocated.
            let total_pages = segment.memsz as usize / arch::PAGE_SIZE + 1;
            let n_pages_from_file = segment.filesz as usize / arch::PAGE_SIZE + 1;
            let n_alloc_pages = total_pages - n_pages_from_file;

            // Calculate the segment's start page in memory.
            let seg_start_page = kfile_start_page + segment.offset as usize;

            // Map pages from the ELF.
            for x in 0..n_pages_from_file {
                let p_offset = x * arch::PAGE_SIZE;
                arch::map(
                    seg_start_page + p_offset as usize,
                    segment.vaddr as usize + p_offset,
                );
            }

            if n_alloc_pages != 0 {
                // Allocate additional pages for the segment.
                let alloc_start_page = env::allocate_pages(n_alloc_pages)
                    .expect("failed to allocate pages to load kernel image");

                // Map remaining pages from allocated pages.
                for x in n_pages_from_file..total_pages {
                    let p_offset = (x - n_pages_from_file) * arch::PAGE_SIZE;
                    let m_offset = x * arch::PAGE_SIZE;
                    arch::map(
                        alloc_start_page + p_offset as usize,
                        segment.vaddr as usize + m_offset,
                    );
                }
            }

            // Zero the memory between filesz and memsz.
            let zeroed_start = segment.vaddr + segment.filesz;
            let zeroed_len = segment.memsz - segment.filesz;
            let _ = unsafe {
                ((*(*ST).boot_services).set_mem)(
                    zeroed_start as *mut core::ffi::c_void,
                    zeroed_len as usize,
                    0,
                )
            };
        }
    }

    elf.entry() as usize
}

// Load a ramdisk into memory from a file, return its start address and length.
pub fn load_ramdisk(rdfile: *mut file::Protocol) -> (usize, usize) {
    // Get the length of the ramdisk file.
    let info_buffer = env::allocate_pool(256).expect("failed to allocate file info buffer");
    let mut finfo_guid = file::INFO_ID;
    let mut size = 256; // whole page available for buffer
    let _ = unsafe {
        ((*rdfile).get_info)(
            rdfile,
            &mut finfo_guid,
            &mut size,
            info_buffer as *mut core::ffi::c_void,
        )
    };
    let rdfile_len = unsafe { (*(info_buffer as *const file::Info)).file_size as usize };
    env::free_pool(info_buffer);

    // Load the ramdisk file contents into memory.
    assert_ne!(rdfile_len, 0, "ramdisk file length must not be zero");
    let n = rdfile_len / arch::PAGE_SIZE + 1;
    let rdfile_start_page = env::allocate_pages(n).expect("failed to allocate ramdisk file pages");
    let _ = unsafe { ((*rdfile).set_position)(rdfile, 0) };
    let status = unsafe {
        ((*rdfile).read)(
            rdfile,
            &mut (rdfile_len as usize),
            rdfile_start_page as *mut core::ffi::c_void,
        )
    };
    if status.is_error() {
        panic!("failed to read contents of ramdisk file");
    }

    (rdfile_start_page, rdfile_len)
}
