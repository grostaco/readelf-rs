use std::{
    fs::OpenOptions,
    io::{Read, Seek, SeekFrom},
    mem::{size_of, transmute, MaybeUninit},
    path::PathBuf,
    slice,
};

use num::PrimInt;

use super::ElfHdr;

#[repr(C)]
#[derive(Debug)]
pub struct ElfShdr<Word, Offset, Addr> {
    sh_name: Word,
    sh_type: Word,
    sh_flags: Addr,
    sh_addr: Addr,
    sh_offset: Offset,
    sh_size: Offset,
    sh_link: Word,
    sh_info: Word,
    sh_addralign: Offset,
    sh_entsize: Offset,
}

impl<W, O, A> ElfShdr<W, O, A>
where
    W: PrimInt,
    O: PrimInt,
    A: PrimInt,
{
    pub fn read<H: PrimInt>(path: &PathBuf) -> Result<Vec<Self>, std::io::Error> {
        let hdr = ElfHdr::<H, W, O, A>::read(path)?;
        let mut file = OpenOptions::new().read(true).open(path)?;
        file.seek(SeekFrom::Start(hdr.shstart().to_u64().unwrap()))?;

        let mut v = Vec::with_capacity(hdr.nsection_headers().to_usize().unwrap());

        for _ in 0..hdr.nsection_headers().to_usize().unwrap() {
            unsafe {
                let mut buf = MaybeUninit::<Self>::uninit();
                file.read_exact(slice::from_raw_parts_mut(
                    transmute(buf.as_mut_ptr()),
                    size_of::<Self>(),
                ))?;

                v.push(buf.assume_init())
            }
        }

        Ok(v)
    }
}
