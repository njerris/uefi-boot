// Definitions and convenience functions for 64-bit ELF files

pub mod program;

use core::mem::size_of;
use core::result::Result;

// Re-export modules to create a flat namespace.
pub use program::*;

/// A set of errors that may arise.
#[derive(Debug)]
pub enum Elf64Error {
    /// The provided slice is too small (usize holds required size).
    SliceTooSmall(usize),
    /// The slice is not an ELF file.
    NotElf,
    /// The slice is not an ELF-64 file.
    NotElf64,
    /// The version of the ELF file is invalid.
    InvalidVersion,
}

/// The possible ABIs specified by the ELF file. Different ABIs may require
/// different interpretations of various fields of ELF structures.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ElfAbi {
    /// A plain ELF, System V ABI with no extensions.
    None,
    Unknown(u8),
    // TODO: add Linux, BSD, etc.
}

impl From<u8> for ElfAbi {
    // Matches a u8 to an ABI.
    fn from(x: u8) -> ElfAbi {
        match x {
            0 => ElfAbi::None,
            _ => ElfAbi::Unknown(x),
        }
    }
}

/// The possible data encoding methods for an ELF file.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ElfData {
    /// 2LSB.
    LittleEndian,
    /// 2MSB.
    BigEndian,
    Unknown(u8),
}

impl From<u8> for ElfData {
    // Matches a u8 to a data encoding method.
    fn from(x: u8) -> ElfData {
        match x {
            1 => ElfData::LittleEndian,
            2 => ElfData::BigEndian,
            _ => ElfData::Unknown(x),
        }
    }
}

/// The possible types for an ELF file.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ElfType {
    None,
    Relocatable,
    Executable,
    SharedObject,
    Core,
    /// Specified by the operating system / environment.
    EnvSpecified(u16),
    /// Specified by the processor type.
    ProcSpecified(u16),
    Unknown,
}

impl From<u16> for ElfType {
    // Matches a u16 to an ELF file type.
    fn from(x: u16) -> ElfType {
        match x {
            0 => ElfType::None,
            1 => ElfType::Relocatable,
            2 => ElfType::Executable,
            3 => ElfType::SharedObject,
            4 => ElfType::Core,
            0xfe00..=0xfeff => ElfType::EnvSpecified(x),
            0xff00..=0xffff => ElfType::ProcSpecified(x),
            _ => ElfType::Unknown,
        }
    }
}

/// The possible machine types for an ELF-64 file.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ElfMachine {
    X86_64,
    AArch64,
    RiscV,
    Unknown(u16),
    // TODO: add other common machine types?
}

impl From<u16> for ElfMachine {
    // Matches a u16 to a machine type.
    fn from(x: u16) -> ElfMachine {
        match x {
            0x3E => ElfMachine::X86_64,
            0xB7 => ElfMachine::AArch64,
            0xF3 => ElfMachine::RiscV,
            _ => ElfMachine::Unknown(x),
        }
    }
}

// The ELF-64 header.
#[repr(C)]
struct Elf64Header {
    ident: [u8; 16],
    type_: u16,
    machine: u16,
    version: u32,
    entry: u64,
    phoff: u64,
    shoff: u64,
    flags: u32,
    ehsize: u16,
    phentsize: u16,
    phnum: u16,
    shentsize: u16,
    shnum: u16,
    shstrndx: u16,
}

/// An ELF-64 object file in memory.
pub struct Elf64<'a>(&'a [u8]);

impl<'a> Elf64<'a> {
    /// Checks a slice to see if it contains a valid ELF-64 header and returns
    /// an Elf64 structure.
    pub fn from_slice(slice: &'a [u8]) -> Result<Elf64, Elf64Error> {
        // The slice must be long enough to contain an ELF-64 header.
        let header_size = size_of::<Elf64Header>();
        if slice.len() < header_size {
            return Err(Elf64Error::SliceTooSmall(header_size));
        }

        // The slice must begin with the ELF magic number.
        if !slice.starts_with(&[0x7f, 0x45, 0x4C, 0x46]) {
            return Err(Elf64Error::NotElf);
        }

        let header: &Elf64Header = unsafe {
            // Safe because we checked that the slice starts with the ELF magic
            // number and has a length of at least the size of an ELF-64 header.
            &*(slice.as_ptr() as *const _ as *const Elf64Header)
        };

        // ident[4] (class) must equal 2 (64 bit).
        if header.ident[4] != 2 {
            return Err(Elf64Error::NotElf64);
        }

        // ident[6] (version) must equal current ELF version (1).
        if header.ident[6] != 1 {
            return Err(Elf64Error::InvalidVersion);
        }

        // Version must equal current ELF version (1).
        if header.version != 1 {
            return Err(Elf64Error::InvalidVersion);
        }

        return Ok(Elf64(slice));
    }

    // Get the header from an Elf64 struct.
    fn header(&self) -> &'a Elf64Header {
        unsafe {
            // Safe because &self refers to a valid Elf64 struct, which can
            // only exist if the slice contains a valid header.
            &*(self.0.as_ptr() as *const _ as *const Elf64Header)
        }
    }

    /// Get the data encoding of the ELF.
    pub fn data(&self) -> ElfData {
        self.header().ident[5].into()
    }

    /// Get the ABI of the ELF.
    pub fn abi(&self) -> ElfAbi {
        self.header().ident[7].into()
    }

    /// Get the ABI version of the ELF.
    pub fn abi_version(&self) -> u8 {
        self.header().ident[8]
    }

    /// Get the file type of the ELF.
    pub fn file_type(&self) -> ElfType {
        self.header().type_.into()
    }

    /// Get the machine type of the ELF.
    pub fn machine(&self) -> ElfMachine {
        self.header().machine.into()
    }

    /// Get the entry point of the ELF.
    pub fn entry(&self) -> u64 {
        self.header().entry
    }

    /// Check if the ELF can run on the current machine.
    pub fn is_valid_locally(&self) -> bool {
        // For x86_64 targets, encoding must be little endian and machine must match.
        #[cfg(target_arch = "x86_64")]
        {
            if self.data() == ElfData::LittleEndian && self.machine() == ElfMachine::X86_64 {
                true
            } else {
                false
            }
        }
    }

    /// Get an iterator over the entries of the program header table.
    pub fn program_headers(&self) -> Result<ProgramHeaderIter, Elf64Error> {
        // Check if the slice is long enough to contain the program header table.
        let ph_size = self.header().phnum * self.header().phentsize;
        let required_size = (self.header().phoff + ph_size as u64) as usize;
        if self.0.len() < required_size {
            return Err(Elf64Error::SliceTooSmall(required_size))
        }

        // In memory, the program header table starts at the address of the ELF
        // buffer plus the offset.
        let start = self.0.as_ptr() as usize + self.header().phoff as usize;
        Ok(ProgramHeaderIter::from_parts(self, start as *const ProgramHeader, self.header().phnum))
    }

    /// Check if the contents of a segment are contained in the file.
    pub fn contains(&self, segment: &'a ProgramHeader) -> bool {
        let required_size = (segment.offset + segment.filesz) as usize;
        if self.0.len() < required_size {
            false
        } else {
            true
        }
    }
}
