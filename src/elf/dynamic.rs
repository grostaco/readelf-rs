use std::io::{Read, Seek};

use super::ElfHdr;

pub struct DynamicRelocs {
    pub name: &'static str,
    pub reloc: DynamicTag,
    pub size: DynamicTag,
    pub rela: RelaState,
}

#[repr(usize)]
pub enum DynamicTag {
    Null,
    Needed,
    PltRelSz,
    PltGod,
    Hash,
    StrTab,
    SymTab,
    Rela,
    RelaSz,
    RelaEnt,
    StrSz,
    SymEnt,
    Init,
    Fini,
    SoName,
    RPath,
    Symbolic,
    Rel,
    RelSz,
    RelEnt,
    PltRel,
    Debug,
    TextRel,
    JmpRel,
    BindNow,
    InitArray,
    FiniArray,
    InitArraySz,
    FiniArraySz,
    RunPath,
    Flags,
    PreInitArray = 32,
    PreInitArraySz,
    SymtabShndx,
    RelrSz,
    RelR,
    RelrEnt,
    Encoding,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum RelaState {
    False,
    True,
    Unknown,
}

pub enum DynValue {
    Val(u64),
    Ptr(u64),
}

pub enum Dyn32Value {
    Val([u8; 4]),
    Ptr([u8; 4]),
}

pub enum Dyn64Value {
    Val([u8; 8]),
    Ptr([u8; 8]),
}

pub struct Dyn {
    pub tag: u64,
    pub value: DynValue,
}

#[repr(C)]
pub struct Elf32Dyn {
    pub tag: [u8; 4],
    pub value: Dyn32Value,
}

#[repr(C)]
pub struct Elf64Dyn {
    pub tag: [u8; 8],
    pub value: Dyn64Value,
}

pub static DYNAMIC_RELOCATIONS: [DynamicRelocs; 3] = [
    DynamicRelocs {
        name: "REL",
        reloc: DynamicTag::Rel,
        size: DynamicTag::RelSz,
        rela: RelaState::False,
    },
    DynamicRelocs {
        name: "RELA",
        reloc: DynamicTag::Rela,
        size: DynamicTag::RelaSz,
        rela: RelaState::True,
    },
    DynamicRelocs {
        name: "PLT",
        reloc: DynamicTag::JmpRel,
        size: DynamicTag::PltRelSz,
        rela: RelaState::Unknown,
    },
];

impl Dyn {
    pub fn read<R: Read + Seek>(file: &mut R, header: &ElfHdr) {}
}
