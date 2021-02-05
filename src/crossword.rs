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
}

#[cfg(test)]
mod tests {
    use super::Crossword;

    #[test]
    fn from_string() {
        let result = Crossword::from_string(String::from("ABCDEFGHI"), 3, 3);

        assert!(result.is_ok());
        let c = result.unwrap();
        assert_eq!(String::from("ABCDEFGHI"), c.contents);
    }
}
