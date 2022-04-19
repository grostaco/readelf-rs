use std::{
    alloc::{alloc, Layout},
    borrow::Borrow,
    io::{self, Read, Seek, SeekFrom},
    mem::size_of,
    slice,
};

use num::{FromPrimitive, ToPrimitive};
use num_derive::FromPrimitive;

use super::{
    hdr::ElfClass, Elf32Addr, Elf32Off, Elf32Word, Elf64Addr, Elf64Off, Elf64Word, Elf64Xword,
    ElfHdr,
};

#[derive(FromPrimitive, PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Copy)]
pub enum ProgramType {
    Null,
    Load,
    Dynamic,
    Interp,
    Note,
    ShLib,
    Phdr,
    Tls,
    LoOS = 0x60000000,
    HiOS = 0x6fffffff,
    LoProc = 0x70000000,
    HiProc = 0x7fffffff,
    GnuEhFrame = 0x60000000 + 0x474e550,
    GnuStack = 0x60000000 + 0x474e551,
    GnuRelro = 0x60000000 + 0x474e552,
    GnuProperty = 0x60000000 + 0x474e553,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug, Copy)]
pub struct ProgramFlags {
    read: bool,
    write: bool,
    execute: bool,
}

#[derive(Debug)]
pub struct ElfPhdr {
    p_type: Elf64Word,
    p_offset: Elf64Off,
    p_vaddr: Elf64Addr,
    p_paddr: Elf64Addr,
    p_flags: Elf64Xword,
    p_memsz: Elf64Xword,
    p_align: Elf64Xword,
}

#[repr(C)]
pub struct Elf32Phdr {
    p_type: Elf32Word,
    p_offset: Elf32Off,
    p_vaddr: Elf32Addr,
    p_paddr: Elf32Addr,
    p_filesz: Elf32Word,
    p_memsz: Elf32Word,
    p_flags: Elf32Word,
    p_align: Elf32Word,
}

#[repr(C)]
pub struct Elf64Phdr {
    p_type: Elf64Word,
    p_flags: Elf64Word,
    p_offset: Elf64Off,
    p_vaddr: Elf64Addr,
    p_paddr: Elf64Addr,
    p_filesz: Elf64Xword,
    p_memsz: Elf64Xword,
    p_align: Elf64Xword,
}

impl ElfPhdr {
    pub fn read<R: Read + Seek>(hdr: &ElfHdr, file: &mut R) -> io::Result<Vec<Self>> {
        file.seek(SeekFrom::Start(hdr.e_phoff)).unwrap();

        let layout = Layout::array::<Elf64Phdr>(hdr.e_phnum as usize).unwrap();
        unsafe {
            let ptr = alloc(layout);

            file.read(slice::from_raw_parts_mut(
                ptr,
                hdr.e_phnum as usize * size_of::<Elf64Phdr>(),
            ))?;

            Ok(match hdr.class().unwrap() {
                ElfClass::ElfClass64 => {
                    (*std::ptr::slice_from_raw_parts(ptr as *const Elf64Phdr, hdr.e_phnum.into()))
                        .iter()
                        .map(ElfPhdr::from)
                        .collect::<Vec<ElfPhdr>>()
                }
                _ => (*std::ptr::slice_from_raw_parts(ptr as *const Elf32Phdr, hdr.e_phnum.into()))
                    .iter()
                    .map(|phdr| phdr.try_into().unwrap())
                    .collect::<Vec<ElfPhdr>>(),
            })
        }
    }

    pub fn program_type(&self) -> Option<ProgramType> {
        ProgramType::from_u32(self.p_type)
    }

    pub fn offset(&self) -> Elf64Off {
        self.p_offset
    }

    pub fn vaddr(&self) -> Elf64Addr {
        self.p_vaddr
    }

    pub fn paddr(&self) -> Elf64Addr {
        self.p_paddr
    }

    pub fn filesz(&self) -> Elf64Xword {
        self.p_memsz
    }

    pub fn flags(&self) -> ProgramFlags {
        ProgramFlags {
            read: self.p_flags & 0x4 == 0x4,
            write: self.p_flags & 0x2 == 0x2,
            execute: self.p_flags & 0x1 == 0x1,
        }
    }

    pub fn align(&self) -> Elf64Xword {
        self.p_align
    }
}

impl From<&Elf64Phdr> for ElfPhdr {
    fn from(phdr: &Elf64Phdr) -> Self {
        let phdr = phdr.borrow();
        Self {
            p_type: phdr.p_type,
            p_offset: phdr.p_offset,
            p_vaddr: phdr.p_vaddr,
            p_paddr: phdr.p_paddr,
            p_memsz: phdr.p_memsz,
            p_flags: phdr.p_flags as u64,
            p_align: phdr.p_align,
        }
    }
}

impl TryFrom<&Elf32Phdr> for ElfPhdr {
    type Error = ();
    fn try_from(phdr: &Elf32Phdr) -> Result<Self, ()> {
        Ok(Self {
            p_type: phdr.p_type,
            p_offset: phdr.p_offset.to_u64().ok_or(())?,
            p_vaddr: phdr.p_vaddr.to_u64().ok_or(())?,
            p_paddr: phdr.p_paddr.to_u64().ok_or(())?,
            p_memsz: phdr.p_memsz.to_u64().ok_or(())?,
            p_flags: phdr.p_flags.to_u64().ok_or(())?,
            p_align: phdr.p_align.to_u64().ok_or(())?,
        })
    }
}

impl ProgramType {
    pub fn display(&self) -> String {
        format!("{:?}", self)
            .chars()
            .fold(String::new(), |mut s, c| {
                if !s.is_empty() && c.is_uppercase() {
                    s.push('_');
                }
                s.push_str(&c.to_uppercase().to_string());
                s
            })
    }
}

impl ProgramFlags {
    pub fn display(&self) -> String {
        let mut s = String::with_capacity(3);
        if self.read {
            s.push('R');
        } else {
            s.push(' ');
        }
        if self.write {
            s.push('W')
        } else {
            s.push(' ');
        }

        if self.execute {
            s.push('E');
        } else {
            s.push(' ');
        }

        s
    }
}
