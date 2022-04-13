use std::{
    fmt::Display,
    fs::OpenOptions,
    io::Read,
    mem::MaybeUninit,
    mem::{size_of, transmute},
    path::Path,
    ptr, slice,
};

use num::PrimInt;

use super::{
    EI_ABIVERSION, EI_CLASS, EI_DATA, EI_MAG0, EI_MAG1, EI_MAG2, EI_MAG3, EI_NINDENT, EI_OSABI,
    EI_VERSION, ELFMAG,
};

macro_rules! get_or_prop {
    () => {};
}

#[derive(Debug)]
#[repr(C)]
pub struct ElfHdr<Half, Word, Offset, Addr> {
    e_ident: [u8; EI_NINDENT],
    e_type: Half,
    e_machine: Half,
    e_version: Word,
    e_entry: Addr,
    e_phoff: Offset,
    e_shoff: Offset,
    e_flags: Word,
    e_ehsize: Half,
    e_phentsize: Half,
    e_phnum: Half,
    e_shentsize: Half,
    e_shnum: Half,
    e_shstrndx: Half,
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

#[derive(Debug)]
pub enum Endian {
    Little,
    Big,
}

impl<H, W, O, A> ElfHdr<H, W, O, A>
where
    H: PrimInt,
    W: PrimInt,
    O: PrimInt,
    A: PrimInt,
{
    pub fn read(path: &Path) -> Result<Self, std::io::Error> {
        unsafe {
            let mut buf = MaybeUninit::<Self>::uninit();
            OpenOptions::new()
                .read(true)
                .open(path)
                .unwrap()
                .read_exact(slice::from_raw_parts_mut(
                    transmute(buf.as_mut_ptr()),
                    size_of::<Self>(),
                ))?;

            Ok(buf.assume_init())
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

    pub fn class(&self) -> u8 {
        self.e_ident[EI_CLASS]
    }

    pub fn endian(&self) -> Option<Endian> {
        match self.e_ident[EI_DATA] {
            0x1 => Some(Endian::Little),
            0x2 => Some(Endian::Big),
            _ => None,
        }
    }

    pub fn machine(&self) -> H {
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
    /*
        pub enum OsABI {
        None,
        SysV,
        HPUX,
        NetBSD,
        Linux,
        Solaris,
        IRIX,
        FreeBSD,
        Tru64,
        ARM,
        Standalone,
    } */

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

/*
pub enum OsABI {
    None,
    SysV,
    HPUX,
    NetBSD,
    Linux,
    Solaris,
    IRIX,
    FreeBSD,
    Tru64,
    ARM,
    Standalone,
}
 */
