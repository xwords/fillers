use std::{fmt, hash::Hash};

#[derive(PartialEq, Eq, Debug, Hash, Clone)]
pub struct Crossword {
    contents: String,
    width: usize,
    height: usize,
}

impl Crossword {
    /// Parses a crossword from a string.
    pub fn from_string(s: String, width: usize, height: usize) -> Result<Crossword, String> {
        let stripped: String = s.chars().filter(|c| *c != '\n').collect();

        if width * height != stripped.len() {
            return Err(String::from("Input dimensions do not match."));
        }
        Ok(Crossword {
            contents: stripped,
            width,
            height,
        })
    }

    pub fn get_entries(&self) -> Vec<EntryLocation> {
        let mut result = vec![];

        let bytes = self.contents.as_bytes();

        let mut start_row = None;
        let mut start_col = None;
        let mut length = 0;
        let mut prefilled = true;

        for row in 0..self.height {
            for col in 0..self.width {
                let c = bytes[row * self.width + col] as char;
                if c != '*' {
                    // White square / letter found.
                    if start_row == None {
                        start_row = Some(row);
                        start_col = Some(col);
                    }
                    length += 1;
                    prefilled &= c != ' ';
                } else {
                    if start_row == None {
                        continue;
                    }

                    let new_entry = EntryLocation {
                        start_row: start_row.unwrap(),
                        start_col: start_col.unwrap(),
                        length,
                        direction: Direction::Across,
                        prefilled,
                    };
                    result.push(new_entry);
                    start_row = None;
                    start_col = None;
                    length = 0;
                    prefilled = true;
                }
            }

            // Process end of row.
            if length > 0 {
                let new_entry = EntryLocation {
                    start_row: start_row.unwrap(),
                    start_col: start_col.unwrap(),
                    length,
                    direction: Direction::Across,
                    prefilled,
                };
                result.push(new_entry);
                start_row = None;
                start_col = None;
                length = 0;
                prefilled = true;
            }
        }

        for col in 0..self.width {
            for row in 0..self.height {
                let c = bytes[row * self.width + col] as char;
                if c != '*' {
                    // White square / letter found.
                    if start_row == None {
                        start_row = Some(row);
                        start_col = Some(col);
                    }
                    length += 1;
                    prefilled &= c != ' ';
                } else {
                    if start_row == None {
                        continue;
                    }

                    let new_entry = EntryLocation {
                        start_row: start_row.unwrap(),
                        start_col: start_col.unwrap(),
                        length,
                        direction: Direction::Across,
                        prefilled,
                    };
                    result.push(new_entry);
                    start_row = None;
                    start_col = None;
                    length = 0;
                    prefilled = true;
                }
            }

            // Process end of row.
            if length > 0 {
                let new_entry = EntryLocation {
                    start_row: start_row.unwrap(),
                    start_col: start_col.unwrap(),
                    length,
                    direction: Direction::Down,
                    prefilled,
                };
                result.push(new_entry);
                start_row = None;
                start_col = None;
                length = 0;
                prefilled = true;
            }
        }

        result
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum Direction {
    Across,
    Down,
}

/// A word location in a `Crossword`.
#[derive(Debug, PartialEq, Clone)]
pub struct EntryLocation {
    pub(crate) start_row: usize,
    pub(crate) start_col: usize,
    pub(crate) length: usize,
    pub(crate) direction: Direction,
    pub(crate) prefilled: bool,
}

impl EntryLocation {
    pub fn new(
        start_row: usize,
        start_col: usize,
        length: usize,
        direction: Direction,
        prefilled: bool,
    ) -> EntryLocation {
        EntryLocation {
            start_row,
            start_col,
            length,
            direction,
            prefilled,
        }
    }
}

#[derive(Clone, Debug)]
pub struct EntryIterator<'s> {
    crossword: &'s Crossword,
    pub(crate) entry_location: &'s EntryLocation,
    index: usize,
}

impl<'s> EntryIterator<'s> {
    pub fn new(crossword: &'s Crossword, entry_location: &'s EntryLocation) -> EntryIterator<'s> {
        EntryIterator {
            crossword,
            entry_location,
            index: 0,
        }
    }
}

impl<'s> Iterator for EntryIterator<'s> {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.entry_location.length {
            return None;
        }

        let char_index = match self.entry_location.direction {
            Direction::Across => {
                self.entry_location.start_row * self.crossword.width
                    + self.entry_location.start_col
                    + self.index
            }
            Direction::Down => {
                (self.entry_location.start_row + self.index) * self.crossword.width
                    + self.entry_location.start_col
            }
        };
        self.index += 1;
        let result = self.crossword.contents.as_bytes()[char_index] as char;
        Some(result)
    }
}

#[cfg(test)]
mod tests {
    use super::{Crossword, EntryIterator};

    #[test]
    fn from_string() {
        let result = Crossword::from_string(String::from("ABCDEFGHI"), 3, 3);

        assert!(result.is_ok());
        let c = result.unwrap();
        assert_eq!(String::from("ABCDEFGHI"), c.contents);
    }

    #[test]
    fn get_entries() {
        let result = Crossword::from_string(String::from("ABCDEFGHIJK*MNOPQRSTUVWX "), 5, 5);

        assert!(result.is_ok());

        let c = result.unwrap();
        let entries = c.get_entries();

        assert_eq!(entries.len(), 12);
        assert_eq!(entries.last().unwrap().prefilled, false);
    }

    #[test]
    fn get_entry_iterator() {
        let result = Crossword::from_string(String::from("ABCDEFGHIJK*MNOPQRSTUVWX "), 5, 5);

        assert!(result.is_ok());

        let c = result.unwrap();
        let entries = c.get_entries();

        let entry_location = entries.first().unwrap();

        let iter = EntryIterator::new(&c, entry_location);
        assert_eq!(String::from("ABCDE"), iter.collect::<String>());
    }
}
