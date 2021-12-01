//! ELF program headers
//!
//! The ELF-64 program header structure provides information required to build a
//! program image in memory from the ELF's contents.

use super::Elf64;

/// Possible types for a program header table entry.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PHType {
    Null,
    /// An entry describing a loadable segment.
    Load,
    /// An entry with dynamic linking information.
    Dynamic,
    /// An entry with an offset to a string describing the requested interpreter.
    Interpreter,
    /// An entry specifying "the location and size of auxiliary information."
    Note,
    /// Reserved, "programs that contain [this type of entry] do not conform to the ABI."
    ShLib,
    /// An entry describing the program header table itself,
    /// "only if the program header table is part of the memory image of the program."
    ProgramHeader,
    /// An entry specifying thread-local storage, "need not supported."
    ThreadLocalStorage,
    /// Specified by the operating system / environment.
    EnvSpecified(u32),
    /// Specified by the processor type.
    ProcSpecified(u32),
    Unknown,
}

impl From<u32> for PHType {
    // Matches a u32 to a program header table entry type.
    fn from(x: u32) -> PHType {
        match x {
            0 => PHType::Null,
            1 => PHType::Load,
            2 => PHType::Dynamic,
            3 => PHType::Interpreter,
            4 => PHType::Note,
            5 => PHType::ShLib,
            6 => PHType::ProgramHeader,
            7 => PHType::ThreadLocalStorage,
            0x60000000..=0x6fffffff => PHType::EnvSpecified(x),
            0x70000000..=0x7fffffff => PHType::ProcSpecified(x),
            _ => PHType::Unknown,
        }
    }
}

/// Possible permissions for an ELF segment.
pub enum SegmentPermissions {
    None,
    R,
    W,
    X,
    RW,
    RX,
    WX,
    RWX,
    Unknown,
}

impl From<u32> for SegmentPermissions {
    // Matches a u32 to a permission set.
    fn from(x: u32) -> SegmentPermissions {
        match x {
            0 => SegmentPermissions::None,
            1 => SegmentPermissions::X,
            2 => SegmentPermissions::W,
            3 => SegmentPermissions::WX,
            4 => SegmentPermissions::R,
            5 => SegmentPermissions::RX,
            6 => SegmentPermissions::RW,
            7 => SegmentPermissions::RWX,
            _ => SegmentPermissions::Unknown,
        }
    }
}

/// An ELF-64 program header table entry.
#[repr(C)]
pub struct ProgramHeader {
    type_: u32,
    flags: u32,
    pub offset: u64,
    pub vaddr: u64,
    pub paddr: u64,
    pub filesz: u64,
    pub memsz: u64,
    pub align: u64,
}

impl ProgramHeader {
    /// Get the type of a program header.
    pub fn type_(&self) -> PHType {
        self.type_.into()
    }
}

/// An iterator over the program headers in the program header table.
pub struct ProgramHeaderIter<'a> {
    _elf: &'a Elf64<'a>,
    first: *const ProgramHeader,
    num: u16,
    current: u16,
}

impl<'a> ProgramHeaderIter<'a> {
    /// Create an iterator over the entries of the program header table.
    pub fn from_parts(e: &'a Elf64, f: *const ProgramHeader, n: u16) -> ProgramHeaderIter<'a> {
        ProgramHeaderIter {
            _elf: e,
            first: f,
            num: n,
            current: 0,
        }
    }
}

impl<'a> Iterator for ProgramHeaderIter<'a> {
    type Item = &'a ProgramHeader;

    fn next(&mut self) -> Option<&'a ProgramHeader> {
        if self.current == self.num {
            None
        } else {
            let ptr = self.first as usize + self.current as usize * core::mem::size_of::<ProgramHeader>();
            self.current += 1;
            unsafe {
                Some(&*(ptr as *const ProgramHeader))
            }
        }
    }
}