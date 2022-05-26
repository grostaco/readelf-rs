use std::{
    fmt::Debug,
    fs::{File, OpenOptions},
    io::{self, Read, Seek, SeekFrom},
    mem::{self, transmute, MaybeUninit},
    path::Path,
    ptr, slice,
};

use num::FromPrimitive;
use num_derive::FromPrimitive;

use super::{
    hdr::ElfClass, Elf32Addr, Elf32Off, Elf32Word, Elf64Addr, Elf64Off, Elf64Word, Elf64Xword,
    ElfHdr,
};

macro_rules! trivial_convert {
    ($self:expr => $field:ident, $variant32:ident, $variant64:ident) => {
        match $self {
            Self::$variant32(variant32) => variant32.$field.into(),
            Self::$variant64(variant64) => variant64.$field.into(),
        }
    };
}

#[derive(Debug, Clone, Copy)]
pub enum ElfShdr {
    Elf32Shdr(Elf32Shdr),
    Elf64Shdr(Elf64Shdr),
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Elf32Shdr {
    name: Elf32Word,
    section_type: Elf32Word,
    flags: Elf32Word,
    addr: Elf32Addr,
    offset: Elf32Off,
    size: Elf32Word,
    link: Elf32Word,
    info: Elf32Word,
    addralign: Elf32Word,
    entsize: Elf32Word,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Elf64Shdr {
    /// The name of this section in index of the string table
    pub name: Elf64Word,
    /// Categorization of the section
    pub section_type: Elf64Word,
    /// Attributes
    pub flags: Elf64Xword,
    /// If the section is in the memory image, it will be the address of the first byte of this section; otherwise zero.
    pub addr: Elf64Addr,
    /// Offset in bytes from the beginning of the file to the first byte of this section
    pub offset: Elf64Off,
    /// This section's size
    pub size: Elf64Xword,
    /// The section header table index link, interpretation depends on the section type
    pub link: Elf64Word,
    /// Extra information dependent on the section type
    pub info: Elf64Word,
    /// Alignment constraints
    pub addralign: Elf64Xword,
    /// The size in bytes per each entry of this section, otherwise 0 if this section does not hold a table
    /// of fixed-sized entries
    pub entsize: Elf64Xword,
}

pub struct ElfShdrIter {
    file: File,
    remaining: usize,
    is_elf64: bool,
}

impl ElfShdr {
    pub fn name(&self) -> Elf64Word {
        trivial_convert!(self => name, Elf32Shdr, Elf64Shdr)
    }

    pub fn section_type(&self) -> Option<SectionType> {
        SectionType::from_u32(trivial_convert!(self => section_type, Elf32Shdr, Elf64Shdr))
    }

    pub fn flags(&self) -> u64 {
        trivial_convert!(self => flags, Elf32Shdr, Elf64Shdr)
    }

    pub fn addr(&self) -> Elf64Addr {
        trivial_convert!(self => addr, Elf32Shdr, Elf64Shdr)
    }

    pub fn offset(&self) -> Elf64Off {
        trivial_convert!(self => offset, Elf32Shdr, Elf64Shdr)
    }

    pub fn size(&self) -> Elf64Xword {
        trivial_convert!(self => size, Elf32Shdr, Elf64Shdr)
    }

    pub fn link(&self) -> Elf64Word {
        trivial_convert!(self => link, Elf32Shdr, Elf64Shdr)
    }

    pub fn info(&self) -> Elf64Word {
        trivial_convert!(self => info, Elf32Shdr, Elf64Shdr)
    }

    pub fn addralign(&self) -> Elf64Xword {
        trivial_convert!(self => addralign, Elf32Shdr, Elf64Shdr)
    }

    pub fn entsize(&self) -> Elf64Xword {
        trivial_convert!(self => entsize, Elf32Shdr, Elf64Shdr)
    }

    pub fn read_string_table<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, io::Error> {
        let hdr = ElfHdr::read(&path)?;
        let index = (hdr.e_shentsize as u64 * hdr.e_shstrndx as u64) + hdr.e_shoff;

        let mut file = OpenOptions::new().read(true).open(&path)?;

        file.seek(SeekFrom::Start(index))?;

        let shdr: Self = match hdr.class().unwrap() {
            ElfClass::None | ElfClass::ElfClass32 => unsafe {
                let mut buf = MaybeUninit::<Elf32Shdr>::uninit();
                file.read_exact(slice::from_raw_parts_mut(
                    transmute(&mut buf),
                    std::mem::size_of::<Elf32Shdr>(),
                ))
                .unwrap();
                buf.assume_init().into()
            },
            ElfClass::ElfClass64 => unsafe {
                let mut buf = MaybeUninit::<Elf64Shdr>::uninit();
                file.read_exact(slice::from_raw_parts_mut(
                    transmute(&mut buf),
                    std::mem::size_of::<Elf64Shdr>(),
                ))
                .unwrap();
                buf.assume_init().into()
            },
        };

        file.seek(SeekFrom::Start(shdr.offset()))?;
        let mut buf = vec![0; shdr.size() as usize];

        file.read(&mut buf)?;

        Ok(buf)
    }

    #[inline]
    pub fn get_string_table<R: Read + Seek>(
        file: &mut R,
        hdr: &ElfHdr,
    ) -> Result<Vec<u8>, std::io::Error> {
        Self::get_data(file, hdr, hdr.e_shstrndx.into(), hdr.e_shoff)
    }

    pub fn get_data<R: Read + Seek>(
        file: &mut R,
        hdr: &ElfHdr,
        index: u64,
        offset: u64,
    ) -> Result<Vec<u8>, std::io::Error> {
        let index = (hdr.e_shentsize as u64 * index) + offset;
        let mut buf = MaybeUninit::<Elf64Shdr>::uninit();

        file.seek(SeekFrom::Start(index))?;

        let shdr: ElfShdr = unsafe {
            file.read(slice::from_raw_parts_mut(
                transmute(&mut buf),
                mem::size_of::<Elf64Shdr>(),
            ))?;

            match hdr.class().unwrap() {
                ElfClass::None | ElfClass::ElfClass32 => {
                    ptr::read(buf.as_ptr() as *const Elf32Shdr).into()
                }

                ElfClass::ElfClass64 => buf.assume_init().into(),
            }
        };

        let mut buf = vec![0; shdr.size() as usize];
        file.seek(SeekFrom::Start(shdr.offset()))?;
        file.read(&mut buf)?;

        Ok(buf)
    }

    pub fn iter<P: AsRef<Path>>(path: P) -> Result<ElfShdrIter, io::Error> {
        let mut file = OpenOptions::new().read(true).open(&path)?;
        let hdr = ElfHdr::read(&path)?;

        let (seek_by, remaining) = (hdr.e_shoff as u64, hdr.e_shnum);
        file.seek(SeekFrom::Start(seek_by))?;

        Ok(ElfShdrIter {
            file,
            remaining: remaining as usize,
            is_elf64: match hdr.class().unwrap() {
                ElfClass::None | ElfClass::ElfClass32 => false,
                ElfClass::ElfClass64 => true,
            },
        })
    }
}

impl From<Elf32Shdr> for ElfShdr {
    fn from(shdr: Elf32Shdr) -> Self {
        Self::Elf32Shdr(shdr)
    }
}

impl From<Elf64Shdr> for ElfShdr {
    fn from(shdr: Elf64Shdr) -> Self {
        Self::Elf64Shdr(shdr)
    }
}

impl Iterator for ElfShdrIter {
    type Item = ElfShdr;
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }

        self.remaining -= 1;
        match self.is_elf64 {
            true => unsafe {
                let mut buf = MaybeUninit::<Elf64Shdr>::uninit();
                self.file
                    .read_exact(slice::from_raw_parts_mut(
                        transmute(&mut buf),
                        std::mem::size_of::<Elf64Shdr>(),
                    ))
                    .unwrap();

                Some(buf.assume_init().into())
            },
            false => unsafe {
                let mut buf = MaybeUninit::<Elf32Shdr>::uninit();
                self.file
                    .read_exact(slice::from_raw_parts_mut(
                        transmute(&mut buf),
                        std::mem::size_of::<Elf32Shdr>(),
                    ))
                    .unwrap();

                Some(buf.assume_init().into())
            },
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

#[repr(usize)]
#[derive(Clone, PartialEq, Eq, Debug, FromPrimitive)]
pub enum SectionType {
    Null = 0x0,
    ProgBits = 0x1,
    SymTab = 0x2,
    StrTab = 0x3,
    Rela = 0x4,
    Hash = 0x5,
    Dynamic = 0x6,
    Note = 0x7,
    NoBits = 0x8,
    Rel = 0x9,
    SHLib = 0xA,
    DynSym = 0xB,
    InitArray = 0xE,
    FiniArray = 0xF,
    PreInitArray = 0x10,
    Group = 0x11,
    SymTabShndx = 0x12,
    LoProc = 0x70000000,
    HiProc = 0x7FFFFFFF,
    LoUser = 0x80000000,
    HiUser = 0xFFFFFFFF,

    // GNU additional section types
    GnuHash = 0x6ffffff6,
    VerDef = 0x6FFFFFFD,
    VerNeed = 0x6FFFFFFE,
    VerSym = 0x6FFFFFFF,
}

#[repr(u64)]
pub enum SectionFlag {
    Write = 1 << 0,
    Alloc = 1 << 1,
    ExecInstr = 1 << 2,
    Merge = 1 << 4,
    Strings = 1 << 5,
    InfoLink = 1 << 6,
    LinkOrder = 1 << 7,
    OsNonConforming = 1 << 8,
    Group = 1 << 9,
    Tls = 1 << 10,
    Exclude = 0x80000000,
    Compressed = 1 << 11,
    GnuMbind = 0x01000000,
}
