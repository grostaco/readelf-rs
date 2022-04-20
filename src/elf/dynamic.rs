use std::{
    io::{self, Read, Seek},
    mem::{size_of, transmute, MaybeUninit},
    ptr, slice,
};

use super::{hdr::ElfClass, ElfHdr};

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

#[repr(C)]
pub enum Dyn32Value {
    Val([u8; 4]),
    Ptr([u8; 4]),
}

#[repr(C)]
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
    pub fn read<R: Read + Seek>(file: &mut R, hdr: &ElfHdr) -> io::Result<Self> {
        let mut buf = MaybeUninit::<Elf64Dyn>::uninit();

        unsafe {
            file.read(slice::from_raw_parts_mut(
                transmute(buf.as_mut_ptr()),
                size_of::<Elf64Dyn>(),
            ))?;

            Ok(match hdr.class().unwrap() {
                ElfClass::ElfClass64 => (&buf.assume_init()).into(),
                _ => (&ptr::read(transmute::<_, *const Elf32Dyn>(&buf.assume_init()))).into(),
            })
        }
    }
}

impl From<&Elf64Dyn> for Dyn {
    fn from(b: &Elf64Dyn) -> Self {
        unsafe {
            Self {
                tag: ptr::read(ptr::addr_of!(b.tag) as *const u64),
                value: match b.value {
                    Dyn64Value::Val(v) => {
                        DynValue::Val(*transmute::<_, *const u64>(&v as *const u8))
                    }
                    Dyn64Value::Ptr(p) => {
                        DynValue::Val(*transmute::<_, *const u64>(&p as *const u8))
                    }
                },
            }
        }
    }
}

impl From<&Elf32Dyn> for Dyn {
    fn from(b: &Elf32Dyn) -> Self {
        unsafe {
            Self {
                tag: ptr::read(ptr::addr_of!(b.tag) as *const u32) as u64,
                value: match b.value {
                    Dyn32Value::Val(v) => {
                        DynValue::Val(*transmute::<_, *const u32>(&v as *const u8) as u64)
                    }
                    Dyn32Value::Ptr(p) => {
                        DynValue::Val(*transmute::<_, *const u32>(&p as *const u8) as u64)
                    }
                },
            }
        }
    }
}
