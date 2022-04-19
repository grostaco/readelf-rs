/*
I have almost no idea what's going on here.
Directly ported from https://github.com/bminor/binutils-gdb/blob/1eeb0316304f2d4e2c48aa8887e28c936bfe4f4d/include/elf/internal.h
*/

use super::{
    phdr::ProgramType,
    shdr::{ElfShdr, SectionFlag, SectionType},
    ElfPhdr,
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
