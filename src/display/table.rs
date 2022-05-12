use std::io;
use std::io::Write;

use termcolor::{Color, ColorSpec, StandardStream, WriteColor};

// Consider using a colored buffer?
pub trait Entry {
    fn header(&self) -> String;
    fn display(&self, stream: &mut StandardStream) -> io::Result<()>;
}

pub struct Table<E> {
    columns: Vec<String>,
    entries: Vec<E>,
}

pub struct Series {}

impl<E> Table<E>
where
    E: Entry,
{
    pub fn new<I, T>(columns: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<String>,
    {
        Self {
            columns: columns.into_iter().map(Into::into).collect(),
            entries: Vec::new(),
        }
    }

    pub fn insert_row(&mut self, entry: E) {
        self.entries.push(entry)
    }

    pub fn numbered_display(&self, stream: &mut StandardStream) -> io::Result<()> {
        if self.entries.is_empty() {
            return Ok(());
        }

        let pad = (self.entries.len().log10() as usize + 1).max(2);

        stream.set_color(ColorSpec::new().set_fg(Some(Color::Blue)))?;
        write!(stream, "  [")?;
        stream.set_color(ColorSpec::new().set_fg(Some(Color::White)))?;
        write!(stream, "{:>pad$}", "Nr", pad = pad)?;
        stream.set_color(ColorSpec::new().set_fg(Some(Color::Blue)))?;
        write!(stream, "] ")?;

        stream.set_color(ColorSpec::new().set_fg(Some(Color::Green)))?;
        writeln!(stream, "{}", self.entries[0].header())?;

        for (i, entry) in self.entries.iter().enumerate() {
            stream.set_color(ColorSpec::new().set_fg(Some(Color::Blue)))?;
            write!(stream, "  [")?;
            stream.set_color(ColorSpec::new().set_fg(Some(Color::White)))?;
            write!(stream, "{:>pad$}", i, pad = pad)?;
            stream.set_color(ColorSpec::new().set_fg(Some(Color::Blue)))?;
            write!(stream, "] ")?;
            stream.reset()?;
            entry.display(stream)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::io;

    use termcolor::{Color, ColorSpec, StandardStream, WriteColor};

    use crate::elf::shdr::ElfShdr;

    use super::{Entry, Table};

    impl Entry for ElfShdr {
        fn header(&self) -> String {
            "Name               Type              Address           Offset\nSize               EntSize           Flags  Link  Info  Align".to_string()
        }

        fn display(&self, s: &mut StandardStream) -> io::Result<()> {
            s.set_color(ColorSpec::new().set_fg(Some(Color::Red)))?;
            // writeln!(s, "{} {}", )?;
            s.reset()?;
            Ok(())
        }
    }

    #[test]
    fn foo() {}
}

/*
print_color!(stdout, Color::Green, " {:18}", "Name");
print_color!(stdout, Color::Green, " {:17}", "Type");
print_color!(stdout, Color::Green, " {:17}", "Address");
print_color!(stdout, Color::Green, " {:16}\n      ", "Offset");

print_color!(stdout, Color::Green, " {:18}", "Size");
print_color!(stdout, Color::Green, " {:17}", "EntSize");
print_color!(stdout, Color::Green, " {:18}", "Flags  Link  Info");
print_color!(stdout, Color::Green, " {:18}", "Align");
 */
