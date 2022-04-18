use std::{
    fs,
    path::{Path, PathBuf},
};

use super::{shdr::ElfShdr, ElfHdr};

pub struct File {
    file_path: PathBuf,
    file: fs::File,
    header: ElfHdr,
    section_headers: Vec<ElfShdr>,
    string_table: Vec<u8>,
}

impl File {
    pub fn new<P>(path: P) -> Result<Self, std::io::Error>
    where
        P: AsRef<Path>,
    {
        let mut file = fs::File::open(&path)?;
        let header = ElfHdr::read_file(&mut file)?;

        let section_headers = ElfShdr::iter(&path)?.collect::<Vec<ElfShdr>>();
        let string_table = ElfShdr::get_string_table(&mut file, &header)?;

        Ok(Self {
            file_path: PathBuf::from(path.as_ref()),
            file,
            header,
            section_headers,
            string_table,
        })
    }

    pub fn header(&self) -> &ElfHdr {
        &self.header
    }

    pub fn section_headers(&self) -> &[ElfShdr] {
        &self.section_headers
    }

    pub fn string_lookup(&self, index: usize) -> Option<String> {
        if index > self.string_table.len() {
            return None;
        }

        Some(
            self.string_table
                .iter()
                .skip(index)
                .take_while(|&&c| c != 0)
                .map(|&c| char::from(c))
                .collect::<String>(),
        )
    }
}
