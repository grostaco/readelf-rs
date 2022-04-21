/*
I have almost no idea what's going on here.
Directly ported from https://github.com/bminor/binutils-gdb/blob/1eeb0316304f2d4e2c48aa8887e28c936bfe4f4d/include/elf/internal.h
This is where all the evil lies
*/

use std::{
    alloc::{alloc, dealloc, Layout},
    io::{self, Read, Seek, SeekFrom},
    mem::{align_of, size_of},
    ptr,
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

// let layout = Layout::from_size_align(dynamic_size, align_of::<Elf64Dyn>()).unwrap();

// unsafe {
//     let mut _ptr = alloc(layout);

//     let buf = ptr::slice_from_raw_parts_mut(_ptr, dynamic_size);

//     file.seek(SeekFrom::Start(dynamic_addr)).unwrap();
//     file.read(&mut *buf)?;

//     let result = Ok(match hdr.class().unwrap() {
//         ElfClass::ElfClass64 => (*ptr::slice_from_raw_parts(
//             _ptr as *const Elf64Dyn,
//             dynamic_size / size_of::<Elf64Dyn>(),
//         ))
//         .iter()
//         .map(Dyn::from)
//         .collect(),
//         _ => (*ptr::slice_from_raw_parts(
//             _ptr as *const Elf32Dyn,
//             dynamic_size / size_of::<Elf32Dyn>(),
//         ))
//         .iter()
//         .map(Dyn::from)
//         .collect(),
//     });

//     dealloc(_ptr, layout);
//     result

// very dangerous function. Unless you are me, please avoid using this or exercise extreme caution (test the code as soon as you use this function)
// A few notes:
//  E64 must be larger than or equal to E32
//  E32 must have equal or less alignment than E64
//  E32 and E64 must be repr(C)
// If something goes wrong, check these ^
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
    let layout = Layout::from_size_align(size_of::<E64>() * nmemb, align_of::<E64>()).unwrap();
    let _ptr = alloc(layout);
    let buf = ptr::slice_from_raw_parts_mut(_ptr, size_of::<E64>() * nmemb);

    file.seek(offset)?;
    let bytes_read = file.read(&mut *buf)?;

    if bytes_read != size_of::<E64>() * nmemb && bytes_read != size_of::<E32>() * nmemb {
        panic!(
            "get_data failed to read equal to the expected E64 and E32, got {} bytes expected {} or {} bytes",
            bytes_read, size_of::<E64>() * nmemb, size_of::<E32>() * nmemb
        );
    }

    let result = Ok(match hdr.class().unwrap() {
        ElfClass::ElfClass64 => (*ptr::slice_from_raw_parts(_ptr as *const E64, nmemb))
            .iter()
            .map(E::from)
            .collect(),
        ElfClass::ElfClass32 => (*ptr::slice_from_raw_parts(_ptr as *const E32, nmemb))
            .iter()
            .map(E::from)
            .collect(),
        ElfClass::None => panic!("Cannot handle ELF header with None class"),
    });

    dealloc(_ptr, layout);
    result
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
