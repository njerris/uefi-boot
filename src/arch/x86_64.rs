// Paging support for x86_64 systems

use crate::env;

// The page size used for mappings.
pub const PAGE_SIZE: usize = 4096;

// The present bit of a page table entry.
const PRESENT: u64 = 1;

// Mask to get a pointed frame from a page table entry.
const FRAME_MASK: u64 = 0x000ffffffffff000;

// Get the PT index for an address.
fn ptl1_index(addr: usize) -> usize {
    (addr >> 12) & 511
}

// Get the PD index for an address.
fn ptl2_index(addr: usize) -> usize {
    (addr >> (12 + 9)) & 511
}

// Get the PDP index for an address.
fn ptl3_index(addr: usize) -> usize {
    (addr >> (12 + 9 + 9)) & 511
}

// Get the PML4 index for an address.
fn ptl4_index(addr: usize) -> usize {
    (addr >> (12 + 9 + 9 + 9)) & 511
}

// Convert a pointer to a page table reference.
fn get_pt_from_ptr(ptr: usize) -> &'static mut [u64; 512] {
    unsafe { &mut *(ptr as *mut [u64; 512]) }
}

// Get the active root page table.
fn get_root_pt() -> &'static mut [u64; 512] {
    let ptr: u64;
    unsafe {
        // Read the cr3 register.
        asm!("mov {0}, cr3", out(reg) ptr);
    }
    get_pt_from_ptr(ptr as usize)
}

// Get a new zeroed page table.
fn get_zeroed_pt() -> usize {
    let page = env::allocate_pages(1).expect("failed to allocate page table");

    let pt = get_pt_from_ptr(page);
    for entry in pt.iter_mut() {
        *entry = 0;
    }

    page
}

// Check if an address is page aligned.
pub fn check_page_alignment(addr: usize) -> bool {
    if addr & 4095 != 0 {
        false
    } else {
        true
    }
}

// Prepare the root page table.
pub fn prepare_root_pt() {
    let ptl4 = get_root_pt();

    // Xero the higher half (entries 256-511).
    for entry in ptl4.iter_mut().skip(256) {
        *entry = 0;
    }
}

// Map a page (panics if overwriting a pre-existing mapping).
// Assumptions:
// 1. The higher-half of the root page table has already been zeroed.
// 2. efiloader makes absolutely no huge page mappings; all mappings are l1 page table entries.
// 3. efiloader only sets the PRESENT bit; the kernel will adjust its own mappings later.
pub fn map(page: usize, addr: usize) {
    assert_eq!(page & 4095, 0, "map requires page aligned addresses");
    assert_eq!(addr & 4095, 0, "map requires page aligned addresses");
    assert!(
        addr >= 0xffff800000000000,
        "efiloader should not map addresses in the lower-half"
    );

    let ptl4 = get_root_pt();
    let ptl4_e = ptl4[ptl4_index(addr)];
    let ptl3;
    if ptl4_e == 0 {
        let ptl3_ptr = get_zeroed_pt();
        ptl4[ptl4_index(addr)] = ptl3_ptr as u64 | PRESENT;
        ptl3 = get_pt_from_ptr(ptl3_ptr);
    } else {
        ptl3 = get_pt_from_ptr((ptl4_e & FRAME_MASK) as usize)
    }

    let ptl3_e = ptl3[ptl3_index(addr)];
    let ptl2;
    if ptl3_e == 0 {
        let ptl2_ptr = get_zeroed_pt();
        ptl3[ptl3_index(addr)] = ptl2_ptr as u64 | PRESENT;
        ptl2 = get_pt_from_ptr(ptl2_ptr);
    } else {
        ptl2 = get_pt_from_ptr((ptl3_e & FRAME_MASK) as usize)
    }

    let ptl2_e = ptl2[ptl2_index(addr)];
    let ptl1;
    if ptl2_e == 0 {
        let ptl1_ptr = get_zeroed_pt();
        ptl2[ptl2_index(addr)] = ptl1_ptr as u64 | PRESENT;
        ptl1 = get_pt_from_ptr(ptl1_ptr);
    } else {
        ptl1 = get_pt_from_ptr((ptl2_e & FRAME_MASK) as usize)
    }

    let ptl1_e = ptl1[ptl1_index(addr)];
    if ptl1_e != 0 {
        panic!(
            "caller called map on address {}, but it is already mapped",
            addr
        );
    }
    ptl1[ptl1_index(addr)] = page as u64 | PRESENT;
}
