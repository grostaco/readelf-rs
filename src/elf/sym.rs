use std::{
    io::{Read, Seek, SeekFrom},
    ptr,
};

use num::ToPrimitive;
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::FromPrimitive;

use super::{
    internal::get_data,
    shdr::{ElfShdr, SectionType},
    Elf32Addr, Elf32Half, Elf32Word, Elf64Addr, Elf64Half, Elf64Word, Elf64Xword, ElfHdr,
};

#[repr(C, packed)]
pub struct Elf32Sym {
    name: Elf32Word,
    value: Elf32Addr,
    size: Elf32Word,
    info: u8,
    other: u8,
    shndx: Elf32Half,
}

#[repr(C, packed)]
pub struct Elf64Sym {
    name: Elf64Word,
    info: u8,
    other: u8,
    shndx: Elf64Half,
    value: Elf64Addr,
    size: Elf64Xword,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct ElfSym {
    name: Elf64Word,
    value: Elf64Addr,
    size: Elf64Xword,
    shndx: Elf64Half,
    info: u8,
    other: u8,
}

#[derive(Debug, FromPrimitive, ToPrimitive)]
pub enum SymbolType {
    NoType,
    Object,
    Func,
    Section,
    File,
    Common,
    Tls,
    Relc,
    SRelc,
    Loos,
    GnuIFunc = 10,
    HiOS = 12,
    LoProc,
    HiProc = 15,
}

#[derive(Debug, FromPrimitive, ToPrimitive)]
pub enum SymbolBinding {
    Local,
    Global = 1,
    Weak = 2,
    Loos = 10,
    HiOS = 12,
    LoPROC = 13,
    HiPROC = 15,
}

#[derive(Debug, FromPrimitive, ToPrimitive)]
pub enum SymbolVis {
    Default,
    Internal,
    Hidden,
    Protected,
}

impl ElfSym {
    pub fn read_symbols<R: Seek + Read>(
        file: &mut R,
        hdr: &ElfHdr,
        shdr: &ElfShdr,
        sections: &[ElfShdr],
    ) -> Option<Vec<Self>> {
        if shdr.size() == 0 {
            return None;
        }

        let nelem = shdr.size() / shdr.entsize();

        let syms = unsafe {
            get_data::<_, Elf32Sym, Elf64Sym, ElfSym>(
                file,
                hdr,
                (shdr.size() / shdr.entsize()) as usize,
                SeekFrom::Start(shdr.offset()),
            )
            .unwrap()
        };

        let symtab_shndx = sections.iter().filter(|shdr| {
            shdr.section_type()
                .map(|ty| ty == SectionType::SymTabShndx)
                .unwrap_or(false)
        });

        // for entry in symtab_shndx {
        //     if entry.link() != shdr
        // }

        todo!()
    }

    pub fn name(&self) -> Elf64Word {
        self.name
    }

    pub fn value(&self) -> Elf64Addr {
        self.value
    }

    pub fn size(&self) -> Elf64Xword {
        self.size
    }

    // pub fn info(&self) -> Option<SymbolType> {
    // }

    pub fn binding(&self) -> Option<SymbolBinding> {
        SymbolBinding::from_u8(self.info >> 4)
    }

    pub fn symbol_type(&self) -> Option<SymbolType> {
        SymbolType::from_u8(self.info & 0xF)
    }

    pub fn visibility(&self) -> Option<SymbolVis> {
        SymbolVis::from_u8(self.other & 0xF)
    }

    pub fn shndx(&self) -> u16 {
        self.shndx
    }

    pub fn other(&self) -> u8 {
        self.other
    }
}

impl SymbolType {
    pub fn display(&self) -> String {
        format!("{:?}", self).to_uppercase()
    }
}

impl SymbolBinding {
    pub fn display(&self) -> String {
        format!("{:?}", self).to_uppercase()
    }
}

impl SymbolVis {
    pub fn display(&self) -> String {
        format!("{:?}", self).to_uppercase()
    }
}

// See https://github.com/rust-lang/rust/issues/82523
impl From<Elf32Sym> for ElfSym {
    fn from(sym: Elf32Sym) -> Self {
        unsafe {
            Self {
                name: sym.name,
                value: ptr::addr_of!(sym.value).read_unaligned().to_u64().unwrap(),
                size: ptr::addr_of!(sym.size).read_unaligned().to_u64().unwrap(),
                shndx: sym.shndx,
                info: sym.info,
                other: sym.other,
            }
        }
    }
}

impl From<&Elf32Sym> for ElfSym {
    fn from(sym: &Elf32Sym) -> Self {
        unsafe {
            Self {
                name: sym.name,
                value: ptr::addr_of!(sym.value).read_unaligned().to_u64().unwrap(),
                size: ptr::addr_of!(sym.size).read_unaligned().to_u64().unwrap(),
                info: sym.info,
                shndx: sym.shndx,
                other: sym.other,
            }
        }
    }
}

impl From<&Elf64Sym> for ElfSym {
    fn from(sym: &Elf64Sym) -> Self {
        Self {
            name: sym.name,
            value: sym.value,
            size: sym.size,
            info: sym.info,
            shndx: sym.shndx,
            other: sym.other,
        }
    }
}

impl From<Elf64Sym> for ElfSym {
    fn from(sym: Elf64Sym) -> Self {
        Self {
            name: sym.name,
            value: sym.value,
            size: sym.size,
            info: sym.info,
            shndx: sym.shndx,
            other: sym.other,
        }
    }
}
