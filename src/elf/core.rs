use std::{
    alloc::{alloc, dealloc, Layout},
    fs,
    io::{self, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    slice,
};

use super::{
    dynamic::{Dyn, DynamicTag, RelaState, DYNAMIC_RELOCATIONS},
    hdr::ElfClass,
    shdr::{ElfShdr, SectionType},
    sym::{Elf32Sym, Elf64Sym, ElfSym},
    ElfHdr, ElfPhdr,
};

type Table = Vec<u8>;
pub struct FileData {
    file_path: PathBuf,
    file: fs::File,
    header: ElfHdr,
    program_headers: Vec<ElfPhdr>,
    section_headers: Vec<ElfShdr>,
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

        Ok(Self {
            file_path: PathBuf::from(path.as_ref()),
            file,
            header,
            program_headers,
            section_headers,
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
        for reloc in &DYNAMIC_RELOCATIONS {
            let is_rela = reloc.rela == RelaState::True;
            let name = reloc.name;

            //let rel_size =
        }
    }

    pub fn dynamic_section(&mut self) {
        let entry: Dyn;
    }
}
