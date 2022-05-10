mod color;

pub trait Entry {
    fn header(&self) -> String;
    fn display(&self, f: fmt::Formatter) -> fmt::Result;
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

    pub fn numbered_display(&self) {
        if self.entries.len() == 0 {
            return;
        }
    }
}

// impl<D> Display for Table<D> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         let d = &self.data[0];
//         d.0;
//         write!(f, "{}", 1)
//     }
// }

// impl Index<usize> for Table {
//     type Output =
//     fn index(&self, index: usize) -> &Self::Output {}
// }

// impl IndexMut<usize> for Table {
//     fn index_mut(&mut self, index: usize) -> &mut Self::Output {}
// }

use std::fmt::{self, Formatter};

#[cfg(test)]
mod test {
    use core::fmt;
    use std::fmt::Formatter;

    use super::{Entry, Table};

    struct Relocs {
        pub name: String,
        pub link: u8,
    }

    impl Entry for Relocs {
        fn header(&self) -> String {
            "AAA".to_string()
        }

        fn display(&self, mut f: Formatter) -> fmt::Result {
            write!(f, "{} {}", self.name, self.link)
        }
    }

    #[test]
    fn foo() {
        let mut table: Table<Relocs> = Table::new(["Name", "Link"]);
        table.insert_row(Relocs {
            name: "A".to_string(),
            link: 8,
        });
    }
}
