use super::{Elf32Addr, Elf32Half, Elf32Word, Elf64Addr, Elf64Word, Elf64Xword};

#[repr(C, packed)]
struct Elf32Sym {
    name: Elf32Word,
    value: Elf32Addr,
    size: Elf32Word,
    info: u8,
    other: u8,
    shndx: Elf32Half,
}

#[repr(C, packed)]
struct Elf64Sym {
    name: Elf64Word,
    info: u8,
    other: u8,
    shndx: Elf64Xword,
    value: Elf64Addr,
    size: Elf64Xword,
}

struct ElfSym {
    name: Elf64Word,
    value: Elf64Addr,
    size: Elf64Xword,
    info: u8,
    other: u8,
}

impl ElfSym {}
