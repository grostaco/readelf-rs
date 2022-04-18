use std::{
    fmt::Debug,
    fs::{File, OpenOptions},
    io::{self, Read, Seek, SeekFrom},
    mem::{transmute, MaybeUninit},
    path::Path,
    slice,
};

use num::{FromPrimitive, ToPrimitive};
use num_derive::FromPrimitive;

use super::{
    hdr::ElfClass, Elf32Addr, Elf32Off, Elf32Word, Elf64Addr, Elf64Off, Elf64Word, Elf64Xword,
    ElfHdr,
};

#[derive(Debug)]
pub struct ElfShdr {
    name: Elf64Word,
    section_type: Elf64Word,
    flags: Elf64Xword,
    addr: Elf64Addr,
    offset: Elf64Off,
    size: Elf64Xword,
    link: Elf64Word,
    info: Elf64Word,
    addralign: Elf64Xword,
    entsize: Elf64Xword,
}

pub struct ElfShdrIter {
    file: File,
    remaining: usize,
    is_elf64: bool,
}
#[repr(C)]
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
#[derive(Debug)]
pub struct Elf64Shdr {
    name: Elf64Word,
    section_type: Elf64Word,
    flags: Elf64Xword,
    addr: Elf64Addr,
    offset: Elf64Off,
    size: Elf64Xword,
    link: Elf64Word,
    info: Elf64Word,
    addralign: Elf64Xword,
    entsize: Elf64Xword,
}

impl ElfShdr {
    pub fn name(&self) -> Elf64Word {
        self.name
    }

    pub fn section_type(&self) -> Option<SectionType> {
        SectionType::from_u32(self.section_type)
    }

    pub fn flags(&self) -> u64 {
        self.flags
    }

    pub fn addr(&self) -> Elf64Addr {
        self.addr
    }

    pub fn offset(&self) -> Elf64Off {
        self.offset
    }

    pub fn size(&self) -> Elf64Xword {
        self.size
    }

    pub fn link(&self) -> Elf64Word {
        self.link
    }

    pub fn info(&self) -> Elf64Word {
        self.info
    }

    pub fn addralign(&self) -> Elf64Xword {
        self.addralign
    }

    pub fn entsize(&self) -> Elf64Xword {
        self.entsize
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
        Self {
            name: shdr.name,
            section_type: shdr.section_type,
            flags: shdr.flags.to_u64().unwrap(),
            addr: shdr.addr.to_u64().unwrap(),
            offset: shdr.offset.to_u64().unwrap(),
            size: shdr.size.to_u64().unwrap(),
            link: shdr.link,
            info: shdr.info,
            addralign: shdr.addralign.to_u64().unwrap(),
            entsize: shdr.entsize.to_u64().unwrap(),
        }
    }
}

impl From<Elf64Shdr> for ElfShdr {
    fn from(shdr: Elf64Shdr) -> Self {
        Self {
            name: shdr.name,
            section_type: shdr.section_type,
            flags: shdr.flags,
            addr: shdr.addr,
            offset: shdr.offset,
            size: shdr.size,
            link: shdr.link,
            info: shdr.info,
            addralign: shdr.addralign,
            entsize: shdr.entsize,
        }
    }
}

impl From<ElfShdr> for Elf64Shdr {
    fn from(shdr: ElfShdr) -> Self {
        Self {
            name: shdr.name,
            section_type: shdr.section_type,
            flags: shdr.flags,
            addr: shdr.addr,
            offset: shdr.offset,
            size: shdr.size,
            link: shdr.link,
            info: shdr.info,
            addralign: shdr.addralign,
            entsize: shdr.entsize,
        }
    }
}

impl TryFrom<ElfShdr> for Elf32Shdr {
    type Error = ();
    fn try_from(value: ElfShdr) -> Result<Self, Self::Error> {
        Ok(Self {
            name: value.name,
            section_type: value.section_type,
            flags: value.flags.to_u32().ok_or(())?,
            addr: value.addr.to_u32().ok_or(())?,
            offset: value.offset.to_u32().ok_or(())?,
            size: value.size.to_u32().ok_or(())?,
            link: value.link,
            info: value.info,
            addralign: value.addralign.to_u32().ok_or(())?,
            entsize: value.entsize.to_u32().ok_or(())?,
        })
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

#[derive(Debug, FromPrimitive)]
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
