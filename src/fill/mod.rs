use crate::{crossword, Crossword};

pub struct Filler<'s> {}

pub trait Fill {
    fn fill(&mut self, crossword: &Crossword) -> Result<Crossword, String>;
}

impl<'s> Filler<'s> {
    pub fn new() -> Filler<'s> {
        Filler {}
    }
}

impl<'s> Fill for Filler<'s> {
    fn fill(&mut self, crossword: &Crossword) -> Result<Crossword, String> {
        let mut candidates = vec![crossword.to_owned()];

        let word_boundaries = crossword.get_entries();

        while let Some(candidate) = candidates.pop() {}

        Ok(crossword.clone())
    }
}
