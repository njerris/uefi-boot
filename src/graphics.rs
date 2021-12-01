// Functions to manipulate UEFI graphics modes

use crate::ST;
use r_efi::efi::protocols::graphics_output;

// Get a pointer to information about the current graphics mode.
pub fn get_mode() -> Option<usize> {
    // Obtain a pointer to graphics output protocol on the first device handle supporting it.
    let mut guid = graphics_output::PROTOCOL_GUID;
    let mut gop = 0 as *mut graphics_output::Protocol;
    let status = unsafe {
        ((*(*ST).boot_services).locate_protocol)(
            &mut guid,
            0 as *mut _,
            &mut gop as *mut _ as *mut *mut core::ffi::c_void,
        )
    };
    if status.is_error() {
        println!("WARNING: locate_protocol for graphics ouput protocol failed");
        return None;
    }

    // Print information about the active graphics mode.
    let ch_res = unsafe { (*(*(*gop).mode).info).horizontal_resolution };
    let cv_res = unsafe { (*(*(*gop).mode).info).vertical_resolution };
    println!("graphics mode is {}p by {}p", ch_res, cv_res);

    let mode = unsafe { (*gop).mode };

    Some(mode as usize)
}

/*
// TODO: finish get_gop_modes, which gets multiple graphics modes
// set the highest-resolution graphics mode on every available graphics device
// return a pointer to an array of pointers to graphics mode structures,
// and the number of those structures
pub fn get_gop_modes(image_handle: efi::Handle) -> (usize, u8) {
    // obtain an array of handles supporting graphics output protocol
    // first try with no buffer to get the size of the buffer we need
    let mut guid = graphics_output::PROTOCOL_GUID;
    let mut h_ptr: usize = 0;
    let mut n: usize = 0;
    let status = unsafe {
        ((*(*ST).boot_services).locate_handle)(
            efi::LocateSearchType::ByProtocol,
            &mut guid,
            0 as *mut _,
            &mut n as *mut _,
            h_ptr as *mut efi::Handle,
        )
    };
    if status == efi::Status::BUFFER_TOO_SMALL {
        // n now contains the size of the buffer we need to allocate
        h_ptr = env::allocate_pool(n).expect("failed to allocate buffer for handle_protocol");
    } else {
        panic!("locate_handle with no buffer did not return BUFFER_TOO_SMALL but should");
    }
    // retry with correctly-sized buffer
    let status = unsafe {
        ((*(*ST).boot_services).locate_handle)(
            efi::LocateSearchType::ByProtocol,
            &mut guid,
            0 as *mut _,
            &mut n as *mut _,
            h_ptr as *mut efi::Handle,
        )
    };
    if status.is_error() {
        panic!("locate_handle round 2 returned an error {:?}", status);
    }

    let num_handles = n / core::mem::size_of::<efi::Handle>();
    println!("found {} handles supporting graphics output protocol", num_handles);
    let handles = unsafe {
        core::slice::from_raw_parts_mut(h_ptr as *mut efi::Handle, num_handles)
    };

    // allocate a slice to store pointers to graphics output protocols
    let g = env::allocate_pool(num_handles * core::mem::size_of::<graphics_output::Protocol>())
        .expect("failed to allocate buffer for graphics output protocols");
    let gops = unsafe {
        core::slice::from_raw_parts_mut(g as *mut graphics_output::Protocol, num_handles)
    };

    for x in 0..num_handles {
        println!("handle supporting graphics output protocol: {:?}", handles[x]);
        // open the graphics output protocol on the handle
        let status = unsafe {
            ((*(*ST).boot_services).open_protocol)(
                handles[x],
                &mut guid,
                &mut gops[x] as *mut _ as *mut *mut core::ffi::c_void,
                image_handle,
                0 as efi::Handle,
                1,
            )
        };
        if status.is_error() {
            panic!("failed to open graphics output protocol on handle {:?}", status);
        }

        // set optimal mode on each gop

        env::free_pool(h_ptr);
        env::free_pool(g);
    }

    // check all the video modes
    for x in 0..m.max_mode {
        let mut size = 0 as usize;
        let mut info = 0 as *mut graphics_output::ModeInformation;
        let status = unsafe {
            ((*gop).query_mode)(gop, x, &mut size, &mut info)
        };
        if status.is_error() {
            println!("NOTE: query mode for mode {} failed", x);
            return None
        }

        let h_res = unsafe {
            (*info).horizontal_resolution
        };
        let v_res = unsafe {
            (*info).vertical_resolution
        };

        println!("found video mode, {}p by {}p", h_res, v_res);
    }

   (0,0)
}*/
