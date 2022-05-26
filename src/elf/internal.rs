/*
I have almost no idea what's going on here.
Directly ported from https://github.com/bminor/binutils-gdb/blob/1eeb0316304f2d4e2c48aa8887e28c936bfe4f4d/include/elf/internal.h
This is where all the evil lies
*/

use std::{
    io::{self, Read, Seek, SeekFrom},
    mem, slice,
};

use super::{
    hdr::ElfClass,
    phdr::ProgramType,
    shdr::{ElfShdr, SectionFlag, SectionType},
    ElfHdr, ElfPhdr,
};

#[inline]
fn elf_tbss_special(shdr: &ElfShdr, segment: &ElfPhdr) -> bool {
    shdr.flags() & SectionFlag::Tls as u64 != 0
        && shdr.section_type().unwrap() == SectionType::NoBits
        && segment.program_type().unwrap() != ProgramType::Tls
}

#[inline]
fn elf_section_size(shdr: &ElfShdr, segment: &ElfPhdr) -> u64 {
    if elf_tbss_special(shdr, segment) {
        0
    } else {
        shdr.size()
    }
}

pub unsafe fn get_data<'a, R: Read + Seek, E32, E64, E>(
    file: &mut R,
    hdr: &ElfHdr,
    nmemb: usize,
    offset: SeekFrom,
) -> io::Result<Vec<E>>
where
    E32: 'static,
    E64: 'static,
    E: From<&'a E32>,
    E: From<&'a E64>,
{
    file.seek(offset)?;

    match hdr.class().unwrap() {
        ElfClass::ElfClass32 => {
            let mut buf = Vec::<E32>::with_capacity(nmemb);
            let buf_ptr = buf.as_mut_ptr();

            file.read_exact(slice::from_raw_parts_mut(
                mem::transmute(buf_ptr),
                nmemb * mem::size_of::<E32>(),
            ))?;

            Ok(slice::from_raw_parts(buf_ptr, nmemb)
                .iter()
                .map(Into::into)
                .collect())
        }
        ElfClass::ElfClass64 => {
            let mut buf = Vec::<E64>::with_capacity(nmemb);
            let buf_ptr = buf.as_mut_ptr();
            file.read_exact(slice::from_raw_parts_mut(
                mem::transmute(buf_ptr),
                nmemb * mem::size_of::<E64>(),
            ))?;

            Ok(slice::from_raw_parts(buf_ptr, nmemb)
                .iter()
                .map(Into::into)
                .collect())
        }
        ElfClass::None => panic!("Unsupported elf class"),
    }
}

pub fn offset_from_vma(phdrs: &[ElfPhdr], vma: u64, size: u64) -> u64 {
    for phdr in phdrs {
        if phdr.program_type().unwrap() != ProgramType::Load {
            continue;
        }

        if vma >= (phdr.vaddr() as i64 & -(phdr.align() as i64)) as u64
            && vma + size <= phdr.vaddr() + phdr.filesz()
        {
            return vma - phdr.vaddr() + phdr.offset();
        }
    }

    vma
}

// Don't touch this unless you know what you are doing
pub fn elf_section_in_segment(
    shdr: &ElfShdr,
    segment: &ElfPhdr,
    check_vma: bool,
    strict: bool,
) -> bool {
    let ptype = segment.program_type().unwrap();
    ((((shdr.flags() & SectionFlag::Tls as u64) != 0)
        && (ptype == ProgramType::Tls
            || ptype == ProgramType::GnuRelro
            || ptype == ProgramType::Load))
        || ((shdr.flags() & SectionFlag::Tls as u64) == 0
            && ptype != ProgramType::Tls
            && ptype != ProgramType::Phdr))
        && !((shdr.flags() & SectionFlag::Alloc as u64) == 0
            && (ptype == ProgramType::Load
                || ptype == ProgramType::Dynamic
                || ptype == ProgramType::GnuEhFrame
                || ptype == ProgramType::GnuRelro
                || ptype >= ProgramType::GnuMbindLo && ptype <= ProgramType::GnuMbindHi))
        && (shdr.section_type().unwrap() == SectionType::NoBits
            || shdr.offset() >= segment.offset()
                && (!strict || shdr.offset() - segment.offset() < segment.filesz())
                && (shdr.offset() - segment.offset() + elf_section_size(shdr, segment)
                    <= segment.filesz()))
        && (!check_vma
            || shdr.flags() & SectionFlag::Alloc as u64 == 0
            || shdr.addr() >= segment.vaddr()
                && (!strict || shdr.addr() - segment.vaddr() <= segment.filesz()))
        && ((ptype != ProgramType::Dynamic && ptype != ProgramType::Note)
            || shdr.size() != 0
            || segment.memsz() == 0
            || (shdr.section_type().unwrap() == SectionType::NoBits
                || shdr.offset() > segment.offset()
                    && (shdr.offset() - segment.offset() < segment.filesz())
                    && (shdr.flags() & SectionFlag::Alloc as u64 == 0
                        || (shdr.addr() > segment.vaddr()
                            && shdr.addr() - segment.vaddr() < segment.memsz()))))
}
