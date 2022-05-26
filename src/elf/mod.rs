#![allow(clippy::unused_io_amount)]

pub mod core;
pub mod dynamic;
pub mod hdr;
pub mod internal;
pub mod phdr;
pub mod shdr;
pub mod sym;
pub mod ver;

pub use hdr::ElfHdr;
pub use phdr::ElfPhdr;
pub use shdr::Elf64Shdr;
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
pub const ELFMAG: &str = "\x7fELF";

pub const ELFVER: u8 = 1;

type Elf32Addr = u32;
type Elf32Half = u16;
type Elf32Off = u32;
type Elf32Sword = i32;
type Elf32Word = u32;

type Elf64Addr = u64;
type Elf64Half = u16;
type Elf64SHalf = i16;
type Elf64Off = u64;
type Elf64Sword = i32;
type Elf64Word = u32;
type Elf64Xword = u64;
type Elf64Sxword = u64;
