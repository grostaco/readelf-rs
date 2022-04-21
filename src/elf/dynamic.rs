use std::{
    alloc::{alloc, dealloc, Layout},
    io::{self, Read, Seek, SeekFrom},
    mem::{align_of, size_of, transmute},
    ptr,
};

use num_derive::FromPrimitive;

use super::{hdr::ElfClass, ElfHdr};

pub struct DynamicRelocs {
    pub name: &'static str,
    pub reloc: DynamicTag,
    pub size: DynamicTag,
    pub rela: RelaState,
}

#[derive(FromPrimitive, Clone, Copy)]
#[repr(usize)]
pub enum DynamicTag {
    Null,
    Needed,
    PltRelSz,
    PltGot,
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

    Valrnglo = 0x6ffffd00,
    GnuFlags1 = 0x6ffffdf4,
    GnuPrelinked = 0x6ffffdf5,
    GnuConflictsz = 0x6ffffdf6,
    GnuLiblistsz = 0x6ffffdf7,
    Checksum = 0x6ffffdf8,
    PltPadSz = 0x6ffffdf9,
    MoveEnt = 0x6ffffdfa,
    MoveSz = 0x6ffffdfb,
    Feature = 0x6ffffdfc,
    Posflag1 = 0x6ffffdfd,
    Syminsz = 0x6ffffdfe,
    SymIEntOrValRNGHI = 0x6ffffdff,
    Addrrnglo = 0x6ffffe00,
    GnuHash = 0x6ffffef5,
    TlsdescPlt = 0x6ffffef6,
    TlsdescGot = 0x6ffffef7,
    GnuConflict = 0x6ffffef8,
    GnuLiblist = 0x6ffffef9,
    Config = 0x6ffffefa,
    Depaudit = 0x6ffffefb,
    Audit = 0x6ffffefc,
    PltPad = 0x6ffffefd,
    MoveTab = 0x6ffffefe,
    SymInfoOrAddrrnGHI = 0x6ffffeff,
    Relacount = 0x6ffffff9,
    Relcount = 0x6ffffffa,
    Flags1 = 0x6ffffffb,
    Verdef = 0x6ffffffc,
    Verdefnum = 0x6ffffffd,
    Verneed = 0x6ffffffe,
    Verneednum = 0x6fffffff,
    Versym = 0x6ffffff0,
    Loproc = 0x70000000,
    Hiproc = 0x7fffffff,
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
            file.read(&mut *buf)?;

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
