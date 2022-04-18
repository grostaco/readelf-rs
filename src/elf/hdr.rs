use std::{
    fmt::Display,
    fs::OpenOptions,
    io::Read,
    mem::MaybeUninit,
    mem::{size_of, transmute},
    path::Path,
    slice,
};

use num::ToPrimitive;
use num_derive::FromPrimitive;
use num_traits::cast::FromPrimitive;

use super::{
    Elf32Addr, Elf32Half, Elf32Off, Elf32Word, Elf64Addr, Elf64Half, Elf64Off, Elf64Word,
    EI_ABIVERSION, EI_CLASS, EI_DATA, EI_MAG0, EI_MAG1, EI_MAG2, EI_MAG3, EI_NINDENT, EI_OSABI,
    EI_VERSION,
};

#[derive(Debug)]
pub struct ElfHdr {
    pub e_ident: [u8; EI_NINDENT],
    pub e_type: Elf64Half,
    pub e_machine: Elf64Half,
    pub e_version: Elf64Word,
    pub e_entry: Elf64Addr,
    pub e_phoff: Elf64Off,
    pub e_shoff: Elf64Off,
    pub e_flags: Elf64Word,
    pub e_ehsize: Elf64Half,
    pub e_phentsize: Elf64Half,
    pub e_phnum: Elf64Half,
    pub e_shentsize: Elf64Half,
    pub e_shnum: Elf64Half,
    pub e_shstrndx: Elf64Half,
}

#[repr(C)]
#[derive(Debug)]
pub struct Elf32Hdr {
    pub e_ident: [u8; EI_NINDENT],
    pub e_type: Elf32Half,
    pub e_machine: Elf32Half,
    pub e_version: Elf32Word,
    pub e_entry: Elf32Addr,
    pub e_phoff: Elf32Off,
    pub e_shoff: Elf32Off,
    pub e_flags: Elf32Word,
    pub e_ehsize: Elf32Half,
    pub e_phentsize: Elf32Half,
    pub e_phnum: Elf32Half,
    pub e_shentsize: Elf32Half,
    pub e_shnum: Elf32Half,
    pub e_shstrndx: Elf32Half,
}

#[repr(C)]
#[derive(Debug)]
pub struct Elf64Hdr {
    pub e_ident: [u8; EI_NINDENT],
    pub e_type: Elf64Half,
    pub e_machine: Elf64Half,
    pub e_version: Elf64Word,
    pub e_entry: Elf64Addr,
    pub e_phoff: Elf64Off,
    pub e_shoff: Elf64Off,
    pub e_flags: Elf64Word,
    pub e_ehsize: Elf64Half,
    pub e_phentsize: Elf64Half,
    pub e_phnum: Elf64Half,
    pub e_shentsize: Elf64Half,
    pub e_shnum: Elf64Half,
    pub e_shstrndx: Elf64Half,
}

pub enum OsABI {
    None,
    HPUX,
    NetBSD,
    Linux,
    Solaris,
    IRIX,
    FreeBSD,
    AIX,
    Novell,
    OpenBSD,
    OpenVMS,
    Tru64,
    Unknown(u8),
}

#[derive(Debug)]
pub enum ObjectType {
    None,
    Rel,
    Exec,
    Dyn,
    Core,
    Loos,
    HIOS,
    LOPROC,
    HIPROC,
}

#[derive(FromPrimitive)]
pub enum ElfClass {
    None,
    ElfClass32,
    ElfClass64,
}
#[derive(Debug)]
pub enum Endian {
    Little,
    Big,
}

impl ElfHdr {
    pub fn read<P: AsRef<Path>>(path: P) -> Result<Self, std::io::Error> {
        unsafe {
            let mut buf = MaybeUninit::<Elf32Hdr>::uninit();
            let mut file = OpenOptions::new().read(true).open(&path).unwrap();
            file.read_exact(slice::from_raw_parts_mut(
                transmute(buf.as_mut_ptr()),
                size_of::<Self>(),
            ))?;

            let hdr = buf.assume_init();
            Ok(match hdr.e_ident[EI_CLASS] {
                1 => Self::upcast_elf32(&hdr),
                2 => {
                    let mut buf = MaybeUninit::<Elf64Hdr>::uninit();
                    let mut file = OpenOptions::new().read(true).open(&path).unwrap();

                    file.read_exact(slice::from_raw_parts_mut(
                        transmute(buf.as_mut_ptr()),
                        size_of::<Elf64Hdr>(),
                    ))?;
                    Self::upcast_elf64(&buf.assume_init())
                }
                _ => panic!("Unrecognized elf class"),
            })
        }
    }

    pub fn read_file<R: Read>(file: &mut R) -> Result<Self, std::io::Error> {
        unsafe {
            let mut buf = MaybeUninit::<Elf64Hdr>::uninit();
            file.read(slice::from_raw_parts_mut(
                transmute(buf.as_mut_ptr()),
                size_of::<Self>(),
            ))?;

            let hdr = buf.as_ptr() as *const Elf32Hdr;

            Ok(match (*hdr).e_ident[EI_CLASS] {
                1 => Self::upcast_elf32(&*hdr),
                2 => Self::upcast_elf64(&*transmute::<_, *const Elf64Hdr>(hdr)),
                _ => panic!("Unrecognized elf class"),
            })
        }
    }

    pub fn ident(&self) -> &[u8] {
        &self.e_ident
    }

    pub fn magic(&self) -> u32 {
        self.e_ident[EI_MAG3] as u32
            | (self.e_ident[EI_MAG2] as u32) << 8
            | (self.e_ident[EI_MAG1] as u32) << 16
            | (self.e_ident[EI_MAG0] as u32) << 24
    }

    pub fn magic_reversed(&self) -> u32 {
        self.e_ident[EI_MAG0] as u32
            | (self.e_ident[EI_MAG1] as u32) << 8
            | (self.e_ident[EI_MAG2] as u32) << 16
            | (self.e_ident[EI_MAG3] as u32) << 24
    }

    pub fn magic_ok(&self) -> bool {
        let endian = match self.endian() {
            Some(e) => e,
            None => return false,
        };

        let x = match endian {
            Endian::Big => self.magic_reversed(),
            Endian::Little => self.magic(),
        };

        x == 0x7f454c46
    }

    pub fn class(&self) -> Option<ElfClass> {
        ElfClass::from_u8(self.e_ident[EI_CLASS])
    }

    pub fn endian(&self) -> Option<Endian> {
        match self.e_ident[EI_DATA] {
            0x1 => Some(Endian::Little),
            0x2 => Some(Endian::Big),
            _ => None,
        }
    }

    pub fn entry(&self) -> Elf64Addr {
        self.e_entry
    }

    pub fn phstart(&self) -> Elf64Off {
        self.e_phoff
    }

    pub fn shstart(&self) -> Elf64Off {
        self.e_shoff
    }

    pub fn nheaders(&self) -> Elf64Half {
        self.e_phnum
    }

    pub fn flags(&self) -> Elf64Word {
        self.e_flags
    }

    pub fn header_size(&self) -> Elf64Half {
        self.e_ehsize
    }

    pub fn section_size(&self) -> Elf64Half {
        self.e_shentsize
    }

    pub fn nsection_headers(&self) -> Elf64Half {
        self.e_shnum
    }

    pub fn program_headers_size(&self) -> Elf64Half {
        self.e_phentsize
    }

    pub fn table_index(&self) -> Elf64Half {
        self.e_shstrndx
    }

    pub fn machine(&self) -> Elf64Half {
        self.e_machine
    }

    pub fn version(&self) -> u8 {
        self.e_ident[EI_VERSION]
    }

    pub fn abi_version(&self) -> u8 {
        self.e_ident[EI_ABIVERSION]
    }

    pub fn os_abi(&self) -> OsABI {
        match self.e_ident[EI_OSABI] {
            0x0 => OsABI::None,
            0x1 => OsABI::HPUX,
            0x2 => OsABI::NetBSD,
            0x3 => OsABI::Linux,
            0x6 => OsABI::Solaris,
            0x8 => OsABI::IRIX,
            0x9 => OsABI::FreeBSD,
            0x0A => OsABI::Tru64,
            0x0B => OsABI::Novell,
            0x0C => OsABI::OpenVMS,
            i => OsABI::Unknown(i),
        }
    }

    pub fn ftype(&self) -> Option<ObjectType> {
        let e_type = match self.e_type.to_u32() {
            Some(e) => e,
            _ => return None,
        };

        match e_type {
            0x0 => Some(ObjectType::None),
            0x1 => Some(ObjectType::Rel),
            0x2 => Some(ObjectType::Exec),
            0x3 => Some(ObjectType::Dyn),
            0x4 => Some(ObjectType::Core),
            0xFE00 => Some(ObjectType::Loos),
            0xFEFF => Some(ObjectType::HIOS),
            0xFF00 => Some(ObjectType::LOPROC),
            0xFFFF => Some(ObjectType::HIPROC),
            _ => None,
        }
    }

    pub fn upcast_elf32(hdr: &Elf32Hdr) -> Self {
        Self {
            e_ident: hdr.e_ident,
            e_type: hdr.e_type,
            e_machine: hdr.e_machine,
            e_version: hdr.e_version,
            e_entry: hdr.e_entry.to_u64().unwrap(),
            e_phoff: hdr.e_phoff.to_u64().unwrap(),
            e_shoff: hdr.e_shoff.to_u64().unwrap(),
            e_flags: hdr.e_flags,
            e_ehsize: hdr.e_ehsize,
            e_phentsize: hdr.e_phentsize,
            e_phnum: hdr.e_phnum,
            e_shentsize: hdr.e_shentsize,
            e_shnum: hdr.e_shnum,
            e_shstrndx: hdr.e_shstrndx,
        }
    }

    pub fn upcast_elf64(hdr: &Elf64Hdr) -> Self {
        Self {
            e_ident: hdr.e_ident,
            e_type: hdr.e_type,
            e_machine: hdr.e_machine,
            e_version: hdr.e_version,
            e_entry: hdr.e_entry,
            e_phoff: hdr.e_phoff,
            e_shoff: hdr.e_shoff,
            e_flags: hdr.e_flags,
            e_ehsize: hdr.e_ehsize,
            e_phentsize: hdr.e_phentsize,
            e_phnum: hdr.e_phnum,
            e_shentsize: hdr.e_shentsize,
            e_shnum: hdr.e_shnum,
            e_shstrndx: hdr.e_shstrndx,
        }
    }

    pub fn downcast_elf32(&self) -> Elf32Hdr {
        todo!()
    }

    pub fn downcast_elf64(&self) -> Elf64Hdr {
        todo!()
    }
}

impl Display for OsABI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::None => "UNIX - System V",
            Self::HPUX => "HP-UX",
            Self::NetBSD => "NetBSD",
            Self::Linux => "Linux",
            Self::Solaris => "Solaris",
            Self::IRIX => "IRIX",
            Self::AIX => "AIX",
            Self::Novell => "Novell Modesto",
            Self::OpenBSD => "OpenBSD",
            Self::OpenVMS => "OpenVMS",
            Self::FreeBSD => "FreeBSD",
            Self::Tru64 => "UNIX - Tru64",
            Self::Unknown(_) => "Unknown",
        })
    }
}
