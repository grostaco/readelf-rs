#![feature(int_log)]

use std::{
    fs::OpenOptions,
    io::{BufRead, BufReader, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    str::FromStr,
};

use clap::Parser;

mod elf;

use num::ToPrimitive;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use crate::elf::{
    hdr::{ElfClass, Endian},
    shdr::{ElfShdr, SectionType},
    ElfHdr, ELFVER,
};

macro_rules! set_color {
    ($stdout:expr, $color:path) => {
        $stdout
            .set_color(ColorSpec::new().set_fg(Some($color)))
            .unwrap();
    };
}

macro_rules! print_color {
    ($stdout:expr, $color:path, $fmt:expr, $($ctx:tt)*) => {
        $stdout
            .set_color(ColorSpec::new().set_fg(Some($color)))
            .unwrap();
        print!($fmt, $($ctx)*);
    };
}

macro_rules! attr_pad {
    ($stdout:expr, $color:path, $attr:expr, $ctx:expr, $pad:expr) => {
        set_color!($stdout, $color);
        print!("{}", $attr);
        set_color!($stdout, Color::White);
        println!(
            ":{ctx:>pad$}",
            pad = $pad - $attr.len() + $ctx.len(),
            ctx = $ctx
        );
    };
}

#[derive(Parser, Debug)]
#[clap(
    author = "Xetera Mnemonics <grostaco@gmail.com>",
    version,
    about = "A simple readelf implementation"
)]
struct Args {
    /// ELF files
    files: Vec<String>,

    /// Equivalent to: -h -l -S -s -r -d -V -A -I
    #[clap(short, long)]
    all: bool,

    /// Display the program header
    #[clap(short = 'h', long = "program-headers")]
    show_headers: bool,

    /// Display the section headers
    #[clap(short = 'S', long = "section-headers", alias = "sections")]
    show_sections: bool,
}

fn main() {
    let args = Args::parse();
    let mut should_pad = false;
    let mut stdout = StandardStream::stdout(ColorChoice::Always);

    for f in args.files {
        if args.show_headers {
            let hdr = ElfHdr::read(Path::new("../ComputerSystems/bin/out")).unwrap();

            set_color!(stdout, Color::Yellow);
            print!("ELF Header");
            set_color!(stdout, Color::Blue);
            println!(" {}", f);
            set_color!(stdout, Color::Magenta);
            print!("Magic");
            set_color!(stdout, Color::White);
            print!(":\t\t");
            for i in hdr.ident() {
                print!(" {:02x}", i);
            }
            print!("\n");
            attr_pad!(
                stdout,
                Color::Green,
                "Class",
                match hdr.class().unwrap() {
                    ElfClass::ElfClass32 => "ELF32",
                    ElfClass::ElfClass64 => "ELF64",
                    ElfClass::None => "Unknown",
                },
                36
            );

            attr_pad!(
                stdout,
                Color::Green,
                "Data",
                match hdr.endian() {
                    Some(Endian::Big) => "2's complement, big endian",
                    Some(Endian::Little) => "2's complement, little endian",
                    _ => "Unknown",
                },
                36
            );

            attr_pad!(
                stdout,
                Color::Green,
                "Version",
                format!(
                    "{} {}",
                    hdr.version(),
                    match hdr.version() {
                        ELFVER => "(current version)",
                        _ => "",
                    }
                ),
                36
            );

            attr_pad!(
                stdout,
                Color::Green,
                "OS/ABI",
                format!("{}", hdr.os_abi()),
                36
            );

            attr_pad!(
                stdout,
                Color::Green,
                "ABI Version",
                format!("{}", hdr.abi_version()),
                36
            );

            attr_pad!(
                stdout,
                Color::Green,
                "Type",
                format!("{:#?}", hdr.ftype().unwrap()),
                36
            );

            attr_pad!(
                stdout,
                Color::Green,
                "Machine",
                format!("{}", hdr.machine()),
                36
            );

            attr_pad!(
                stdout,
                Color::Green,
                "Entry point addresss",
                format!("0x{:x}", hdr.entry()),
                36
            );

            attr_pad!(
                stdout,
                Color::Green,
                "Start of program headers",
                format!("{} (bytes into file)", hdr.phstart()),
                36
            );

            attr_pad!(
                stdout,
                Color::Green,
                "Start of section headers",
                format!("{} (bytes into file)", hdr.shstart()),
                36
            );

            attr_pad!(
                stdout,
                Color::Green,
                "Flags",
                format!("{}", hdr.flags()),
                36
            );

            attr_pad!(
                stdout,
                Color::Green,
                "Size of this header",
                format!("{} (bytes)", hdr.header_size()),
                36
            );

            attr_pad!(
                stdout,
                Color::Green,
                "Size of program headers",
                format!("{} (bytes)", hdr.program_headers_size()),
                36
            );

            attr_pad!(
                stdout,
                Color::Green,
                "Number of program headers",
                format!("{}", hdr.nheaders()),
                36
            );

            attr_pad!(
                stdout,
                Color::Green,
                "Size of program headers",
                format!("{} (bytes)", hdr.section_size()),
                36
            );

            attr_pad!(
                stdout,
                Color::Green,
                "Number of section headers",
                format!("{}", hdr.nsection_headers()),
                36
            );

            attr_pad!(
                stdout,
                Color::Green,
                "Section header string table index",
                format!("{}", hdr.table_index()),
                36
            );

            should_pad = true;
        }

        if args.show_sections {
            if should_pad {
                println!("");
            }
            print_color!(stdout, Color::Yellow, "{}\n  ", "Section Headers");

            print_color!(stdout, Color::Blue, "{}", "[");
            print_color!(stdout, Color::White, "{}", "Nr");
            print_color!(stdout, Color::Blue, "{}", "]");

            print_color!(stdout, Color::Green, " {:18}", "Name");
            print_color!(stdout, Color::Green, " {:18}", "Type");
            print_color!(stdout, Color::Green, " {:18}", "Address");
            print_color!(stdout, Color::Green, " {:18}\n      ", "Offset");

            print_color!(stdout, Color::Green, " {:18}", "Size");
            print_color!(stdout, Color::Green, " {:18}", "EntSize");
            print_color!(stdout, Color::Green, " {:18}", "Flags  Link  Info");
            print_color!(stdout, Color::Green, " {:18}", "Align");

            let it_shdr = ElfShdr::iter(&f).unwrap();
            let table = ElfShdr::read_string_table(&f).unwrap();

            let max_pad = it_shdr.size_hint().0.log10() as usize + 1;

            for (i, shdr) in it_shdr.enumerate() {
                print_color!(stdout, Color::Blue, "{}", "\n  [");
                print_color!(
                    stdout,
                    Color::White,
                    "{i:max_pad$}",
                    i = i,
                    max_pad = max_pad
                );

                print_color!(stdout, Color::Blue, "{}", "] ");
                set_color!(stdout, Color::White);
                print!(
                    "{:18}",
                    table
                        .iter()
                        .skip(shdr.name() as usize)
                        .take(16 + 1)
                        .take_while(|&&c| c != 0)
                        .map(|c| *c as char)
                        .collect::<String>()
                );

                // print!(
                //     " {:18}",
                //     match shdr.sh_type {
                //         SectionType::NULL => "NULL",
                //         _ => "?",
                //     }
                // );
            }
            println!("");
        }
    }

    //println!("{:#?}", hdr);
}
