use std::{mem::size_of, path::Path};

use clap::Parser;

mod elf;
use elf::ElfHdr64;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use crate::elf::{hdr::Endian, ELFVER};

macro_rules! set_color {
    ($stdout:expr, $color:path) => {
        $stdout
            .set_color(ColorSpec::new().set_fg(Some($color)))
            .unwrap();
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
    #[clap(short = 'l', long = "program-headers")]
    show_headers: bool,
}

fn main() {
    let args = Args::parse();
    let mut stdout = StandardStream::stdout(ColorChoice::Always);

    if args.show_headers {
        let hdr = ElfHdr64::read(Path::new("../ComputerSystems/bin/out")).unwrap();

        set_color!(stdout, Color::Yellow);
        println!("ELF Header");
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
            match hdr.class() {
                1 => "ELF32",
                2 => "ELF64",
                _ => "Unknown",
            },
            16
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
            16
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
            16
        );

        attr_pad!(
            stdout,
            Color::Green,
            "OS/ABI",
            format!("{}", hdr.os_abi()),
            16
        );

        attr_pad!(
            stdout,
            Color::Green,
            "ABI Version",
            format!("{}", hdr.abi_version()),
            16
        );

        attr_pad!(
            stdout,
            Color::Green,
            "Type",
            format!("{:#?}", hdr.ftype().unwrap()),
            16
        );

        attr_pad!(
            stdout,
            Color::Green,
            "Machine",
            format!("{:#?}", hdr.machine()),
            16
        );

        // set_color!(stdout, Color::Green);
        // print!("Class");
        // set_color!(stdout, Color::White);

        // println!(":{:>pad}", "A");
    }
    //println!("{:#?}", hdr);
}
