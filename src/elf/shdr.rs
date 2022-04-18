use std::{
    fmt::Debug,
    fs::{File, OpenOptions},
    io::{self, Read, Seek, SeekFrom},
    marker::PhantomData,
    mem::{transmute, transmute_copy, MaybeUninit},
    path::{Path, PathBuf},
    slice,
};

use num::{FromPrimitive, PrimInt, ToPrimitive};
use num_derive::FromPrimitive;

use super::{
    hdr::ElfClass, Elf32Addr, Elf32Off, Elf32Word, Elf64Addr, Elf64Off, Elf64Word, Elf64Xword,
    ElfHdr,
};

#[repr(C)]
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

// #[repr(C)]
// pub struct ElfNShdr<XWord, Word, Offset, Address> {
//     name: Word,
//     section_type: Word,
//     flags: XWord,
//     addr: Address,
//     offset: Offset,
//     size: XWord,
//     link: Word,
//     info: Word,
//     addralign: XWord,
//     entsize: XWord,
// }

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

    pub fn flags(&self) -> Option<SectionFlag> {
        SectionFlag::from_u64(self.flags)
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

    pub fn downcast_elf32(&self) -> Option<Elf32Shdr> {
        Some(Elf32Shdr {
            name: self.name,
            section_type: self.section_type,
            flags: self.flags.to_u32()?,
            addr: self.addr.to_u32()?,
            offset: self.offset.to_u32()?,
            size: self.size.to_u32()?,
            link: self.link,
            info: self.info,
            addralign: self.addralign.to_u32()?,
            entsize: self.entsize.to_u32()?,
        })
    }

    pub fn downcast_elf64(&self) -> Elf64Shdr {
        unsafe { transmute_copy(self) }
    }

    pub fn upcast_elf32(shdr: &Elf32Shdr) -> Self {
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

    fn upcast_elf64(shdr: &Elf64Shdr) -> Self {
        unsafe { transmute_copy(shdr) }
    }

    pub fn read_string_table<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, io::Error> {
        let hdr = ElfHdr::read(&path)?;
        let index = (hdr.e_shentsize as u64 * hdr.e_shstrndx as u64) + hdr.e_shoff;

        let mut file = OpenOptions::new().read(true).open(&path)?;

        file.seek(SeekFrom::Start(index))?;

        let shdr = match hdr.class().unwrap() {
            ElfClass::None | ElfClass::ElfClass32 => unsafe {
                let mut buf = MaybeUninit::<Elf32Shdr>::uninit();
                file.read_exact(slice::from_raw_parts_mut(
                    transmute(&mut buf),
                    std::mem::size_of::<Elf32Shdr>(),
                ))
                .unwrap();
                ElfShdr::upcast_elf32(&buf.assume_init())
            },
            ElfClass::ElfClass64 => unsafe {
                let mut buf = MaybeUninit::<Elf64Shdr>::uninit();
                file.read_exact(slice::from_raw_parts_mut(
                    transmute(&mut buf),
                    std::mem::size_of::<Elf64Shdr>(),
                ))
                .unwrap();
                ElfShdr::upcast_elf64(&buf.assume_init())
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

                Some(ElfShdr::upcast_elf64(&buf.assume_init()))
            },
            false => unsafe {
                let mut buf = MaybeUninit::<Elf32Shdr>::uninit();
                self.file
                    .read_exact(slice::from_raw_parts_mut(
                        transmute(&mut buf),
                        std::mem::size_of::<Elf32Shdr>(),
                    ))
                    .unwrap();

                Some(ElfShdr::upcast_elf32(&buf.assume_init()))
            },
        }
    }
}

#[derive(FromPrimitive)]
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
    LoProc = 0x70000000,
    HiProc = 0x7FFFFFFF,
    LoUser = 0x80000000,
    HiUser = 0xFFFFFFFF,

    // GNU additional section types
    VerDef = 0x6FFFFFFD,
    VerNeed = 0x6FFFFFFE,
    VerSym = 0x6FFFFFFF,
}

#[derive(FromPrimitive)]
pub enum SectionFlag {
    Write = 0x1,
    Alloc = 0x2,
    ExecInstr = 0x4,
    RelaLivePatch = 0x100000,
    RoAfterInit = 0x200000,
    MaskProc = 0xf00000,
}

// #[repr(C)]
// #[derive(Debug)]
// pub struct ElfShdr<Word, Offset, Addr> {
//     pub sh_name: Word,
//     pub sh_type: Word,
//     pub sh_flags: Addr,
//     pub sh_addr: Addr,
//     pub sh_offset: Offset,
//     pub sh_size: Offset,
//     pub sh_link: Word,
//     pub sh_info: Word,
//     pub sh_addralign: Offset,
//     pub sh_entsize: Offset,
// }

// pub struct ElfShdrSet(PathBuf);
// pub struct ElfShdrIter<Word, Offset, Addr> {
//     file: File,
//     phantom: PhantomData<ElfShdr<Word, Offset, Addr>>,
// }

// impl ElfShdrSet {
//     pub fn new<P>(path: P) -> Self
//     where
//         PathBuf: From<P>,
//     {
//         Self(PathBuf::from(path))
//     }
//     pub fn iter<Word, Offset, Addr>(
//         &self,
//     ) -> Result<ElfShdrIter<Word, Offset, Addr>, std::io::Error> {
//         Ok(ElfShdrIter {
//             file: File::open(self.0)?,
//             phantom: PhantomData,
//         })
//     }
// }

// impl<W, O, A> Iterator for ElfShdrIter {
//     type Item = ElfShdr<W, O, A>;

//     fn next(&mut self) -> Option<Self::Item> {}
// }

// use std::{
//     fmt::Debug,
//     fs::{File, OpenOptions},
//     io::{Read, Seek, SeekFrom},
//     marker::PhantomData,
//     mem::{size_of, transmute, MaybeUninit},
//     path::PathBuf,
//     ptr, slice,
// };

// use num::PrimInt;

// use super::ElfHdr;

// #[repr(C)]
// #[derive(Debug)]
// pub struct ElfShdr<Word, Offset, Addr> {
//     pub sh_name: Word,
//     pub sh_type: Word,
//     pub sh_flags: Addr,
//     pub sh_addr: Addr,
//     pub sh_offset: Offset,
//     pub sh_size: Offset,
//     pub sh_link: Word,
//     pub sh_info: Word,
//     pub sh_addralign: Offset,
//     pub sh_entsize: Offset,
// }

// pub struct ElfShdrIter<W, O, A> {
//     pub file: File,
//     pub remaining: usize,
//     phantom: PhantomData<ElfShdr<W, O, A>>,
// }

// macro_rules! as_u64 {
//     ($x:expr) => {
//         $x.to_u64().unwrap()
//     };
// }

// impl<W, O, A> ElfShdr<W, O, A>
// where
//     W: PrimInt,
//     O: PrimInt,
//     A: PrimInt,
// {
//     pub fn read_string_table<H: PrimInt>(path: &PathBuf) -> Result<Vec<u8>, std::io::Error> {
//         let hdr = ElfHdr::<H, W, O, A>::read(path)?;
//         let mut file = OpenOptions::new().read(true).open(path)?;
//         file.seek(SeekFrom::Start(
//             as_u64!(hdr.e_shentsize) * as_u64!(hdr.e_shstrndx) + as_u64!(hdr.e_shoff),
//         ))?;

//         let shdr = unsafe {
//             let mut buf = MaybeUninit::<Self>::uninit();
//             file.read_exact(slice::from_raw_parts_mut(
//                 transmute(buf.as_mut_ptr()),
//                 size_of::<Self>(),
//             ))?;

//             buf.assume_init()
//         };

//         let mut buf = vec![0; shdr.sh_size.to_usize().unwrap()];

//         file.seek(SeekFrom::Start(shdr.sh_offset.to_u64().unwrap()))
//             .unwrap();
//         file.read(&mut buf).unwrap();

//         Ok(buf)
//     }

//     pub fn read<H: PrimInt>(path: &PathBuf) -> Result<Self, std::io::Error> {
//         let hdr = ElfHdr::<H, W, O, A>::read(path)?;
//         let mut file = OpenOptions::new().read(true).open(path)?;

//         file.seek(SeekFrom::Start(hdr.shstart().to_u64().unwrap()))?;

//         unsafe {
//             let mut buf = MaybeUninit::<Self>::uninit();
//             file.read_exact(slice::from_raw_parts_mut(
//                 transmute(buf.as_mut_ptr()),
//                 size_of::<Self>(),
//             ))?;

//             Ok(buf.assume_init())
//         }
//     }
// }

// impl<W, O, A> ElfShdrIter<W, O, A>
// where
//     W: PrimInt,
//     O: PrimInt,
//     A: PrimInt,
// {
//     pub fn new<H: PrimInt>(path: &PathBuf, offset: u64) -> Result<Self, std::io::Error> {
//         let mut file = OpenOptions::new().read(true).open(path)?;
//         let hdr = ElfHdr::<H, W, O, A>::read(path)?;

//         if offset != 0 {
//             file.seek(SeekFrom::Start(offset))?;
//         }
//         Ok(Self {
//             file,
//             remaining: hdr.e_shnum.to_usize().unwrap(),
//             phantom: PhantomData,
//         })
//     }
// }

// impl<W, O, A> Iterator for ElfShdrIter<W, O, A> {
//     type Item = ElfShdr<W, O, A>;
//     fn next(&mut self) -> Option<Self::Item> {
//         if self.remaining == 0 {
//             return None;
//         }

//         self.remaining -= 1;
//         unsafe {
//             let mut buf = MaybeUninit::<ElfShdr<W, O, A>>::uninit();
//             self.file
//                 .read_exact(slice::from_raw_parts_mut(
//                     transmute(buf.as_mut_ptr()),
//                     size_of::<ElfShdr<W, O, A>>(),
//                 ))
//                 .unwrap();
//             Some(buf.assume_init())
//         }
//     }

//     fn size_hint(&self) -> (usize, Option<usize>) {
//         (self.remaining, Some(self.remaining))
//     }
// }
