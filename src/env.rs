// Functions to interact with the UEFI boot services environment

use crate::{ROOT, ST};
use core::fmt::{self, Write};
use r_efi::efi;

// A struct representing the EFI console for print! and println!
// NOTE: Because write_str invokes the EFI for each individual character, it
// is very slow. But, since it's used minimally, it doesn't matter.
pub struct Conout;

impl Write for Conout {
    fn write_str(&mut self, string: &str) -> Result<(), fmt::Error> {
        for c in string.chars() {
            let _ = unsafe {
                ((*(*ST).con_out).output_string)(
                    (*ST).con_out,
                    [c as u16, 0].as_ptr() as *mut efi::Char16,
                )
            };
            if c == '\n' {
                let _ = unsafe {
                    ((*(*ST).con_out).output_string)(
                        (*ST).con_out,
                        ['\r' as u16, 0].as_ptr() as *mut efi::Char16,
                    )
                };
            }
        }

        Ok(())
    }
}

pub fn _print(args: fmt::Arguments) {
    Conout.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::env::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => (print!("\n"));
    ($fmt:expr) => (print!(concat!($fmt, "\n")));
    ($fmt:expr, $($arg:tt)*) => (print!(concat!($fmt, "\n"), $($arg)*));
}

// Initialize the filesystem.
pub fn init_fs(image_handle: efi::Handle) {
    // Open the loaded image protocol.
    let mut loaded_image_p = 0 as *mut efi::protocols::loaded_image::Protocol;
    let mut guid = efi::protocols::loaded_image::PROTOCOL_GUID;
    let status = unsafe {
        ((*(*ST).boot_services).open_protocol)(
            image_handle,
            &mut guid,
            &mut loaded_image_p as *mut _ as *mut *mut core::ffi::c_void,
            image_handle,
            0 as efi::Handle,
            1,
        )
    };
    if status.is_error() {
        panic!("open_protocol: loaded image protocol {:?}", status);
    }

    // Open the simple file system protocol on the device that efiloader was loaded from.
    let mut file_system_p = 0 as *mut efi::protocols::simple_file_system::Protocol;
    guid = efi::protocols::simple_file_system::PROTOCOL_GUID;
    let status = unsafe {
        ((*(*ST).boot_services).open_protocol)(
            (*loaded_image_p).device_handle,
            &mut guid,
            &mut file_system_p as *mut _ as *mut *mut core::ffi::c_void,
            image_handle,
            0 as efi::Handle,
            1,
        )
    };
    if status.is_error() {
        panic!("open_protocol: simple file system protocol {:?}", status);
    }

    // Open the simple file system volume, store in the ROOT file descriptor.
    let status = unsafe {
        ((*file_system_p).open_volume)(
            file_system_p,
            (&mut ROOT) as *mut *mut efi::protocols::file::Protocol,
        )
    };
    if status.is_error() {
        panic!("open volume {:?}", status);
    }
}

// Allocate memory from the pool.
pub fn allocate_pool(s: usize) -> Option<usize> {
    let mut ptr = 0 as *mut core::ffi::c_void;
    let status = unsafe {
        ((*(*ST).boot_services).allocate_pool)(
            efi::LOADER_DATA,
            s,
            &mut ptr as *mut *mut _,
        )
    };
    if status.is_error() {
        None
    } else {
        Some(ptr as usize)
    }
}

// Free pool memory.
pub fn free_pool(buffer: usize) {
    let status = unsafe { ((*(*ST).boot_services).free_pool)(buffer as *mut core::ffi::c_void) };
    if status.is_error() {
        panic!("called free_pool() on invalid buffer: {}", buffer);
    }
}

// Allocate physical pages.
pub fn allocate_pages(n: usize) -> Option<usize> {
    let mut page: efi::PhysicalAddress = 0;
    let status = unsafe {
        ((*(*ST).boot_services).allocate_pages)(
            efi::ALLOCATE_ANY_PAGES,
            efi::LOADER_DATA,
            n,
            &mut page,
        )
    };
    if status.is_error() {
        None
    } else {
        Some(page as usize)
    }
}

// Open a file in read-only mode.
pub fn open_file(path: *mut u16) -> Option<*mut efi::protocols::file::Protocol> {
    let mut file = 0 as *mut efi::protocols::file::Protocol;
    let status = unsafe {
        ((*ROOT).open)(
            ROOT,
            (&mut file) as *mut *mut efi::protocols::file::Protocol,
            path,
            efi::protocols::file::MODE_READ,
            0,
        )
    };
    if status.is_error() {
        println!("ERROR: open_file {:?}", status);
        None
    } else {
        Some(file)
    }
}
