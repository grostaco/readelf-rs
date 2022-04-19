use std::ptr;

use num::ToPrimitive;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use super::{Elf32Addr, Elf32Half, Elf32Word, Elf64Addr, Elf64Half, Elf64Word, Elf64Xword};

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

#[derive(FromPrimitive)]
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

impl ElfSym {
    pub fn name(&self) -> Elf64Word {
        self.name
    }

    pub fn value(&self) -> Elf64Addr {
        self.value
    }

    pub fn size(&self) -> Elf64Xword {
        self.size
    }

    pub fn info(&self) -> Option<SymbolType> {
        SymbolType::from_u8(self.info)
    }

    pub fn shndx(&self) -> u16 {
        self.shndx
    }

    pub fn other(&self) -> u8 {
        self.other
    }
}

// See https://github.com/rust-lang/rust/issues/82523
impl TryFrom<Elf32Sym> for ElfSym {
    type Error = ();

    fn try_from(sym: Elf32Sym) -> Result<Self, ()> {
        unsafe {
            Ok(Self {
                name: sym.name,
                value: ptr::addr_of!(sym.value)
                    .read_unaligned()
                    .to_u64()
                    .ok_or(())?,
                size: ptr::addr_of!(sym.size)
                    .read_unaligned()
                    .to_u64()
                    .ok_or(())?,
                shndx: sym.shndx,
                info: sym.info,
                other: sym.other,
            })
        }
    }
}

impl TryFrom<&Elf32Sym> for ElfSym {
    type Error = ();

    fn try_from(sym: &Elf32Sym) -> Result<Self, ()> {
        unsafe {
            Ok(Self {
                name: sym.name,
                value: ptr::addr_of!(sym.value)
                    .read_unaligned()
                    .to_u64()
                    .ok_or(())?,
                size: ptr::addr_of!(sym.size)
                    .read_unaligned()
                    .to_u64()
                    .ok_or(())?,
                info: sym.info,
                shndx: sym.shndx,
                other: sym.other,
            })
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
