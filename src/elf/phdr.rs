#[repr(C)]
pub struct ElfPhdr<Word, Offset, Addr> {
    p_type: Word,
    p_offset: Offset,
    p_vaddr: Addr,
    p_padder: Addr,
    p_memsz: Word,
    p_flags: Word,
    p_align: Word,
}
