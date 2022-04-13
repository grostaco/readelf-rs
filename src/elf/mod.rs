pub mod hdr;
pub mod phdr;

pub use hdr::ElfHdr;
pub use phdr::ElfPhdr;

// Constants taken directly from linux/elf.h

pub const EI_NINDENT: usize = 16;

pub const EI_MAG0: usize = 0;
pub const EI_MAG1: usize = 1;
pub const EI_MAG2: usize = 2;
pub const EI_MAG3: usize = 3;

pub const EI_CLASS: usize = 4;
pub const EI_DATA: usize = 5;
pub const EI_VERSION: usize = 6;
pub const EI_OSABI: usize = 7;
pub const EI_ABIVERSION: usize = 8;
pub const EI_PAD: usize = 9;

pub const ELFMAG0: usize = 0x7f;
pub const ELFMAG1: usize = 'E' as usize;
pub const ELFMAG2: usize = 'L' as usize;
pub const ELFMAG3: usize = 'F' as usize;
pub const ELFMAG: &'static str = "\x7fELF";

pub const ELFVER: u8 = 1;

// As ElfHdr and ElfPhdr are not packed to avoid unaligned references, care should be taken
// as to not introduce padding for different generic types
// See https://github.com/rust-lang/rust/issues/82523

pub type ElfHdr32 = hdr::ElfHdr<u16, u32, u32, u32>;
pub type ElfPhdr32 = phdr::ElfPhdr<u32, u32, u32>;
pub type ElfHdr64 = hdr::ElfHdr<u16, u32, u64, u64>;
pub type ElfPhdr64 = phdr::ElfPhdr<u64, u64, u64>;
