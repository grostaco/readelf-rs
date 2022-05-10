use std::{
    alloc::{alloc, dealloc, Layout},
    fs,
    io::{self, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    slice,
};

use super::{
    dynamic::{Dyn, DynamicTag},
    hdr::ElfClass,
    internal::get_data,
    phdr::ProgramType,
    shdr::{ElfShdr, SectionType},
    sym::{Elf32Sym, Elf64Sym, ElfSym},
    ElfHdr, ElfPhdr,
};

use num_traits::FromPrimitive;

type Table = Vec<u8>;
pub struct FileData {
    file_path: PathBuf,
    file: fs::File,
    header: ElfHdr,
    program_headers: Vec<ElfPhdr>,
    section_headers: Vec<ElfShdr>,
    dynamic_addr: u64,
    dynamic_size: usize,
    dynamic_info: [u64; DynamicTag::Encoding as usize],
    string_table: Vec<u8>,
}

impl FileData {
    pub fn new<P>(path: P) -> Result<Self, std::io::Error>
    where
        P: AsRef<Path>,
    {
        let mut file = fs::File::open(&path)?;
        let header = ElfHdr::read_file(&mut file)?;

        let program_headers = ElfPhdr::read(&header, &mut file).unwrap();
        let section_headers = ElfShdr::iter(&path)?.collect::<Vec<ElfShdr>>();
        let string_table = ElfShdr::get_string_table(&mut file, &header)?;

        let (dynamic_addr, dynamic_size) = match program_headers
            .iter()
            .find(|phdr| phdr.program_type().unwrap() == ProgramType::Dynamic)
        {
            Some(phdr) => (phdr.offset(), phdr.filesz() as usize),
            None => (0, 0usize),
        };

        Ok(Self {
            file_path: PathBuf::from(path.as_ref()),
            file,
            header,
            program_headers,
            section_headers,
            dynamic_addr,
            dynamic_size,
            dynamic_info: [0; 38],
            string_table,
        })
    }

    pub fn header(&self) -> &ElfHdr {
        &self.header
    }

    pub fn section_headers(&self) -> &[ElfShdr] {
        &self.section_headers
    }

    pub fn program_headers(&self) -> &[ElfPhdr] {
        &self.program_headers
    }

    pub fn dynamic_symbols(&mut self) -> Option<io::Result<Vec<ElfSym>>> {
        if let Some(dyn_section) = self.section_headers.iter().find(|shdr| {
            shdr.section_type()
                .map_or(false, |stype| stype == SectionType::DynSym)
        }) {
            let syms = ElfSym::read_symbols(
                &mut self.file,
                &self.header,
                dyn_section,
                &self.section_headers,
            )?;

            return Some(syms);
        }

        None
    }

    // Please for the love of god someone rewrite this
    // This is a powder keg waiting to explode
    pub fn table_symbols(&mut self) -> io::Result<Vec<(String, Table, Vec<ElfSym>)>> {
        let sym_sections = self.section_headers.iter().filter(|shdr| {
            shdr.section_type()
                .map(|st| st == SectionType::SymTab || st == SectionType::DynSym)
                .unwrap_or(false)
        });

        let mut v = Vec::new();

        for shdr in sym_sections {
            let table = if shdr.link() == self.header.table_index().into() {
                ElfShdr::get_string_table(&mut self.file, &self.header)
            } else {
                ElfShdr::get_data(
                    &mut self.file,
                    &self.header,
                    shdr.link() as u64,
                    self.header.e_shoff,
                )
            }
            .unwrap();

            let name = self.string_lookup(shdr.name() as usize).unwrap();

            self.file.seek(SeekFrom::Start(shdr.offset()))?;

            let buf = unsafe {
                let layout =
                    Layout::array::<Elf64Sym>((shdr.size() / shdr.entsize()) as usize).unwrap();

                let ptr = alloc(layout);
                let slice = slice::from_raw_parts_mut(ptr, shdr.size() as usize);

                self.file.read(slice)?;

                let buf = match self.header.class().unwrap() {
                    ElfClass::ElfClass32 => (*std::ptr::slice_from_raw_parts(
                        ptr as *const Elf32Sym,
                        (shdr.size() / shdr.entsize()) as usize as usize,
                    ))
                    .iter()
                    .map(|sym| sym.try_into().unwrap())
                    .collect(),
                    ElfClass::ElfClass64 => (*std::ptr::slice_from_raw_parts(
                        ptr as *const Elf64Sym,
                        (shdr.size() / shdr.entsize()) as usize as usize,
                    ))
                    .iter()
                    .map(|sym| sym.into())
                    .collect::<Vec<ElfSym>>(),
                    _ => panic!("Unsupported elf type"),
                };

                dealloc(ptr, layout);

                buf
            };

            v.push((name, table, buf));
        }

        Ok(v)
    }

    pub fn string_lookup_iter(&self, index: usize) -> Option<impl Iterator<Item = char> + '_> {
        if index > self.string_table.len() {
            return None;
        }
        Some(
            self.string_table
                .iter()
                .skip(index)
                .take_while(|&&c| c != 0)
                .map(|&c| char::from(c)),
        )
    }

    #[inline]
    pub fn string_lookup(&self, index: usize) -> Option<String> {
        self.string_lookup_iter(index).map(|it| it.collect())
    }

    pub fn relocations(&mut self) -> io::Result<Vec<(String, Table, Vec<ElfSym>)>> {
        let sym_sections = self
            .section_headers
            .iter()
            .filter(|shdr| {
                shdr.section_type()
                    .map(|st| st == SectionType::Rela)
                    .unwrap_or(false)
            })
            .map(|shdr| self.string_lookup(shdr.name() as usize).unwrap())
            .collect::<String>();

        println!("{}", sym_sections);

        todo!()
    }

    pub fn process_relocs(&mut self) {
        self.process_dynamic_section();

        for shdr in self.section_headers.iter().filter(|shdr| {
            matches!(
                shdr.section_type().unwrap(),
                SectionType::Rela | SectionType::Rel
            )
        }) {
            print!("\nRelocation section ");
            print!("{}", self.string_lookup(shdr.name() as usize).unwrap());

            let rel_offset = shdr.offset();
            let rel_size = shdr.size();
            let num_rela = rel_size / shdr.entsize();

            println!(
                " at offset 0x{:x} contains {} entries:",
                rel_offset, num_rela
            );

            if shdr.link() != 0 && shdr.link() < self.header().e_shnum.into() {
                let symsec = self.section_headers()[shdr.link() as usize];
                if !matches!(
                    symsec.section_type().unwrap(),
                    SectionType::SymTab | SectionType::DynSym
                ) {
                    continue;
                }

                println!("{}", self.string_lookup(symsec.name() as usize).unwrap());

                let table = ElfShdr::get_data(
                    &mut self.file,
                    &self.header,
                    symsec.link().into(),
                    self.header.e_shoff,
                )
                .unwrap();

                let syms = unsafe {
                    get_data::<_, Elf32Sym, Elf64Sym, ElfSym>(
                        &mut self.file,
                        &self.header,
                        (shdr.size() / shdr.entsize()) as usize,
                        SeekFrom::Start(symsec.offset()),
                    )
                    .unwrap()
                };

                for sym in syms {
                    println!(
                        "{:#?}",
                        table
                            .iter()
                            .skip(sym.name() as usize)
                            .take_while(|&&p| p != 0)
                            .map(|i| *i as char)
                            .collect::<String>()
                    );
                }
            }

            if shdr.link() != 0 && shdr.link() < self.header.e_shnum.into() {
                ElfSym::read_symbols(&mut self.file, &self.header, shdr, &self.section_headers);
            }
        }

        // for reloc in &DYNAMIC_RELOCATIONS {
        //     let is_rela = reloc.rela == RelaState::True;
        //     let name = reloc.name;

        //     let rel_size = self.dynamic_info[reloc.size as usize];
        //     let rel_offset = self.dynamic_info[reloc.reloc as usize];

        //     println!("\nRelocation section");

        //     self.string_lookup()

        // println!(
        //     "{} {} {} {}",
        //     reloc.size as usize, reloc.reloc as usize, rel_size, rel_offset
        // );
    }

    pub fn process_dynamic_section(&mut self) {
        let dynamic_section = self.dynamic_section();

        for entry in &dynamic_section {
            if entry.tag == DynamicTag::SymTab as u64 {
                self.dynamic_info[DynamicTag::SymTab as usize] = unsafe { entry.value.val };
            }

            if entry.tag == DynamicTag::StrTab as u64 {
                self.dynamic_info[DynamicTag::StrTab as usize] = unsafe { entry.value.val };
            }

            match DynamicTag::from_u64(entry.tag).unwrap() {
                DynamicTag::Null
                | DynamicTag::Needed
                | DynamicTag::PltGot
                | DynamicTag::Hash
                | DynamicTag::StrTab
                | DynamicTag::Rela
                | DynamicTag::RelaSz
                | DynamicTag::Init
                | DynamicTag::Fini
                | DynamicTag::SoName
                | DynamicTag::RPath
                | DynamicTag::Symbolic
                | DynamicTag::Rel
                | DynamicTag::Debug
                | DynamicTag::TextRel
                | DynamicTag::JmpRel
                | DynamicTag::RunPath
                | DynamicTag::PltRelSz
                | DynamicTag::RelaEnt
                | DynamicTag::RelEnt => {
                    self.dynamic_info[entry.tag as usize] = unsafe { entry.value.val }
                }
                _ => {}
            }
        }
    }

    pub fn dynamic_section(&mut self) -> Vec<Dyn> {
        let mut dyns = Dyn::read(
            &mut self.file,
            &self.header,
            self.dynamic_addr,
            self.dynamic_size,
        )
        .unwrap();

        dyns.drain(..)
            .take_while(|d| d.tag != DynamicTag::Null as u64)
            .collect()
    }
}
