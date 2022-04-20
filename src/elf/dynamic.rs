use std::{
    alloc::{alloc, dealloc, Layout},
    io::{self, Read, Seek, SeekFrom},
    mem::{align_of, size_of, transmute},
    ptr,
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

pub union DynValue {
    pub val: u64,
    pub ptr: u64,
}

#[repr(C)]
pub union Dyn32Value {
    val: [u8; 4],
    ptr: [u8; 4],
}

#[repr(C)]
pub union Dyn64Value {
    val: [u8; 8],
    ptr: [u8; 8],
}

pub struct Dyn {
    pub tag: u64,
    pub value: DynValue,
}

#[repr(C, packed)]
pub struct Elf32Dyn {
    pub tag: [u8; 4],
    pub value: Dyn32Value,
}

#[repr(C, packed)]
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
    pub fn read<R: Read + Seek>(
        file: &mut R,
        hdr: &ElfHdr,
        dynamic_addr: u64,
        dynamic_size: usize,
    ) -> io::Result<Vec<Self>> {
        let layout = Layout::from_size_align(dynamic_size, align_of::<Elf64Dyn>()).unwrap();

        unsafe {
            let mut _ptr = alloc(layout);

            let buf = ptr::slice_from_raw_parts_mut(_ptr, dynamic_size);

            file.seek(SeekFrom::Start(dynamic_addr)).unwrap();
            let bytes_read = file.read(&mut *buf)?;
            println!(
                "Read {} bytes from {} expected with type size {}",
                bytes_read,
                dynamic_size,
                size_of::<Elf64Dyn>()
            );

            let result = Ok(match hdr.class().unwrap() {
                ElfClass::ElfClass64 => (*ptr::slice_from_raw_parts(
                    _ptr as *const Elf64Dyn,
                    dynamic_size / size_of::<Elf64Dyn>(),
                ))
                .iter()
                .map(Dyn::from)
                .collect(),
                _ => (*ptr::slice_from_raw_parts(
                    _ptr as *const Elf32Dyn,
                    dynamic_size / size_of::<Elf32Dyn>(),
                ))
                .iter()
                .map(Dyn::from)
                .collect(),
            });

            dealloc(_ptr, layout);
            result
        }
    }
}

impl From<&Elf64Dyn> for Dyn {
    fn from(b: &Elf64Dyn) -> Self {
        unsafe {
            Self {
                tag: ptr::read(ptr::addr_of!(b.tag) as *const u64),
                value: DynValue {
                    val: *transmute::<_, *const u64>(ptr::addr_of!(b.value) as *const u8),
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
                value: DynValue {
                    val: *transmute::<_, *const u32>(ptr::addr_of!(b.value) as *const u8) as u64,
                },
            }
        }
    }
}
