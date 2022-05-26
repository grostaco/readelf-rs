use std::io::{Read, Seek};

use super::{
    shdr::{ElfShdr, SectionType},
    Elf32Half, Elf32Word, Elf64Half, Elf64Word, ElfHdr,
};

// The version structures are currently size independent. Elf32Verdef is identical to Elf64Verdef

#[repr(C)]
pub struct Elf32Verdef {
    version: Elf32Half,
    flags: Elf32Half,
    ndx: Elf32Half,
    cnt: Elf32Half,
    hash: Elf32Word,
    aux: Elf32Word,
    next: Elf32Word,
}

#[repr(C)]
pub struct Elf64Verdef {
    version: Elf64Half,
    flags: Elf64Half,
    ndx: Elf64Half,
    cnt: Elf64Half,
    hash: Elf64Word,
    aux: Elf64Word,
    next: Elf64Word,
}

#[derive(Debug, Clone, Copy)]
pub struct ElfVerdef {
    version: Elf64Half,
    flags: Elf64Half,
    ndx: Elf64Half,
    cnt: Elf64Half,
    hash: Elf64Word,
    aux: Elf64Word,
    next: Elf64Word,
}

impl ElfVerdef {
    pub fn read<R: Seek + Read>(
        mut _file: R,
        _header: &ElfHdr,
        shdrs: &[ElfShdr],
    ) -> Option<Vec<Self>> {
        let versym = match shdrs
            .iter()
            .find(|shdr| shdr.section_type().unwrap() == SectionType::VerSym)
        {
            Some(versym) => versym,
            _ => return None,
        };

        let _n = versym.size() / versym.entsize();

        // match header.class().unwrap() {
        //     ElfClass::ElfClass64 => ReadBuf::file.read_buf,
        //     _ => {}
        // }
        // let versym = unsafe {
        //     get_data::<_, Elf32Half, Elf64Half, Elf64Half>(
        //         &mut file,
        //         header,
        //         (versym.size() / versym.entsize()) as usize,
        //         SeekFrom::Start(versym.offset()),
        //     )
        //     .unwrap()
        // };

        // Some(verdef)

        todo!()
    }
}

impl From<Elf32Verdef> for ElfVerdef {
    fn from(verdef: Elf32Verdef) -> Self {
        Self {
            version: verdef.version,
            flags: verdef.flags,
            ndx: verdef.ndx,
            cnt: verdef.cnt,
            hash: verdef.hash,
            aux: verdef.aux,
            next: verdef.next,
        }
    }
}

impl From<Elf64Verdef> for ElfVerdef {
    fn from(verdef: Elf64Verdef) -> Self {
        Self {
            version: verdef.version,
            flags: verdef.flags,
            ndx: verdef.ndx,
            cnt: verdef.cnt,
            hash: verdef.hash,
            aux: verdef.aux,
            next: verdef.next,
        }
    }
}

impl From<&Elf32Verdef> for ElfVerdef {
    fn from(verdef: &Elf32Verdef) -> Self {
        Self {
            version: verdef.version,
            flags: verdef.flags,
            ndx: verdef.ndx,
            cnt: verdef.cnt,
            hash: verdef.hash,
            aux: verdef.aux,
            next: verdef.next,
        }
    }
}

impl From<&Elf64Verdef> for ElfVerdef {
    fn from(verdef: &Elf64Verdef) -> Self {
        Self {
            version: verdef.version,
            flags: verdef.flags,
            ndx: verdef.ndx,
            cnt: verdef.cnt,
            hash: verdef.hash,
            aux: verdef.aux,
            next: verdef.next,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::elf::core::FileData;

    #[test]
    fn foo() {
        let _f = FileData::new("../ComputerSystems/bin/out").unwrap();
        // ElfVerdef::read(&f.)
    }
}
