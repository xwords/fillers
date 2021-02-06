use crossword::{Direction, EntryIterator, EntryLocation};
use rustc_hash::FxHashMap;

use crate::{crossword, index::Index, Crossword};

pub struct Filler<'s> {
    index: &'s Index,
}

pub trait Fill {
    fn fill(&mut self, crossword: &Crossword) -> Result<Crossword, String>;
}

impl<'s> Filler<'s> {
    pub fn new(index: &'s Index) -> Filler<'s> {
        Filler { index }
    }
}

impl<'s> Fill for Filler<'s> {
    fn fill(&mut self, crossword: &Crossword) -> Result<Crossword, String> {
        let mut candidates = vec![crossword.to_owned()];

        let entry_locations = crossword.get_entries();
        let entry_location_lookup = build_square_to_entry_lookup(&entry_locations);

        while let Some(candidate) = candidates.pop() {
            // Find the next entry to fill, sorted by # possible words and start position.
            let to_fill = entry_locations
                .iter()
                .map(|entry_location| EntryIterator::new(&candidate, entry_location))
                .filter(|iter| iter.clone().any(|c| c == ' '))
                .min_by_key(|iter| (iter.entry_location.start_row, iter.entry_location.start_col))
                .unwrap();
        }

        Err(String::from("Failed to fill."))
    }
}

pub fn build_square_to_entry_lookup<'s>(
    entry_locations: &'s [EntryLocation],
) -> FxHashMap<(Direction, usize, usize), &'s EntryLocation> {
    let mut result = FxHashMap::default();

    for entry_location in entry_locations {
        match entry_location.direction {
            Direction::Across => {
                for i in 0..entry_location.length {
                    result.insert(
                        (
                            Direction::Across,
                            entry_location.start_row,
                            entry_location.start_col + i,
                        ),
                        entry_location,
                    );
                }
            }
            Direction::Down => {
                for i in 0..entry_location.length {
                    result.insert(
                        (
                            Direction::Across,
                            entry_location.start_row + i,
                            entry_location.start_col,
                        ),
                        entry_location,
                    );
                }
            }
        }
    }

    result
}
