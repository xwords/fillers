use std::{
    collections::{HashMap, HashSet},
    hash::{self, BuildHasherDefault},
};

use crossword::{Direction, EntryIterator, EntryLocation};
use hash::{Hash, Hasher};
use rustc_hash::{FxHashMap, FxHashSet, FxHasher};

use crate::{crossword, index::Index, Crossword};

pub mod cache;
use cache::{CachedIsValid, CachedWords};
use serde::{Deserialize, Serialize};

pub struct Filler<'s> {
    index: &'s Index,
    is_valid_cache: CachedIsValid,
    word_cache: CachedWords,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct EntryLocationToFill {
    pub(crate) start_row: usize,
    pub(crate) start_col: usize,
    pub(crate) direction: Direction,
}

pub trait Fill {
    fn fill(
        &mut self,
        crossword: &Crossword,
        entry_locations_to_fill: Option<&Vec<EntryLocationToFill>>,
    ) -> Result<Crossword, String>;
}

impl<'s> Filler<'s> {
    pub fn new(index: &'s Index) -> Filler<'s> {
        Filler {
            index,
            is_valid_cache: CachedIsValid::default(),
            word_cache: CachedWords::default(),
        }
    }
}

pub fn is_valid_grid(
    candidate: &Crossword,
    entry_locations: &[&EntryLocation],
    index: &Index,
    mut used_words: FxHashSet<u64>,
    is_valid_cache: &mut CachedIsValid,
) -> (bool, FxHashSet<u64>) {
    for entry_location in entry_locations {
        let iter = EntryIterator::new(candidate, entry_location);

        let mut hasher = FxHasher::default();
        let mut full = true;
        for c in iter.clone() {
            c.hash(&mut hasher);
            full &= c != ' ';
        }
        let key = hasher.finish();

        if full && used_words.contains(&key) {
            return (false, used_words);
        }

        used_words.insert(key);

        if !entry_location.prefilled && !is_valid_cache.is_valid(iter, index) {
            return (false, used_words);
        }
    }
    (true, used_words)
}

pub fn get_orthogonal_words<'s>(
    entry_location: &'s EntryLocation,
    entry_location_lookup: &HashMap<
        (Direction, usize, usize),
        &'s EntryLocation,
        BuildHasherDefault<FxHasher>,
    >,
) -> Vec<&'s EntryLocation> {
    let mut result = Vec::with_capacity(entry_location.length);

    match entry_location.direction {
        Direction::Across => {
            for i in 0..entry_location.length {
                result.push(
                    *entry_location_lookup
                        .get(&(
                            Direction::Down,
                            entry_location.start_row,
                            entry_location.start_col + i,
                        ))
                        .unwrap(),
                );
            }
        }
        Direction::Down => {
            for i in 0..entry_location.length {
                result.push(
                    *entry_location_lookup
                        .get(&(
                            Direction::Across,
                            entry_location.start_row + i,
                            entry_location.start_col,
                        ))
                        .unwrap(),
                );
            }
        }
    }

    result
}

/// Fill a single word in a candidate crossword.
pub fn fill_one_word(candidate: &Crossword, chars: &EntryIterator, word: &str) -> Crossword {
    let entry_location = chars.entry_location;
    let mut result_contents = String::with_capacity(candidate.contents.len());
    let mut word_iter = word.chars();

    match entry_location.direction {
        Direction::Across => {
            for (i, c) in candidate.contents.chars().enumerate() {
                let row = i / candidate.width;
                let col = i % candidate.width;

                if row == entry_location.start_row
                    && entry_location.start_col <= col
                    && col < entry_location.start_col + entry_location.length
                {
                    result_contents.push(word_iter.next().unwrap());
                } else {
                    result_contents.push(c);
                }
            }
        }
        Direction::Down => {
            for (i, c) in candidate.contents.chars().enumerate() {
                let row = i / candidate.width;
                let col = i % candidate.width;

                if col == entry_location.start_col
                    && entry_location.start_row <= row
                    && row < entry_location.start_row + entry_location.length
                {
                    result_contents.push(word_iter.next().unwrap());
                } else {
                    result_contents.push(c);
                }
            }
        }
    }

    Crossword {
        contents: result_contents,
        ..*candidate
    }
}

impl<'s> Fill for Filler<'s> {
    fn fill(
        &mut self,
        crossword: &Crossword,
        entry_locations_to_fill: Option<&Vec<EntryLocationToFill>>,
    ) -> Result<Crossword, String> {
        let mut num_candidates = 0;
        let mut candidates = vec![crossword.to_owned()];

        let entry_locations = crossword.get_entries();
        let entry_location_lookup = build_square_to_entry_lookup(&entry_locations);

        let mut used_words = HashSet::with_capacity_and_hasher(
            entry_locations.len(),
            BuildHasherDefault::<FxHasher>::default(),
        );

        let mut entry_locations_to_fill_lookup = FxHashSet::default();
        if entry_locations_to_fill.is_some() {
            for entry_location in entry_locations_to_fill.unwrap() {
                entry_locations_to_fill_lookup.insert((
                    entry_location.direction.clone(),
                    entry_location.start_row,
                    entry_location.start_col,
                ));
            }
        }
        let partial_fill = entry_locations_to_fill.is_some();

        while let Some(candidate) = candidates.pop() {
            num_candidates += 1;
            // Find the next entry to fill, sorted by # possible words and start position.
            let to_fill_options = entry_locations
                .iter()
                .filter(|iter| {
                    !partial_fill
                        || entry_locations_to_fill_lookup.contains(&(
                            (*iter).direction.clone(),
                            (*iter).start_row,
                            (*iter).start_col,
                        ))
                })
                .map(|entry_location| EntryIterator::new(&candidate, entry_location))
                .filter(|iter| iter.clone().any(|c| c == ' '));

            let entries_left = to_fill_options.clone().count();

            let to_fill_option = to_fill_options.min_by_key(|iter| {
                (
                    self.word_cache.words(iter.clone(), self.index).len(),
                    iter.entry_location.start_row,
                    iter.entry_location.start_col,
                )
            });

            if !to_fill_option.is_some() {
                return Ok(candidate);
            }

            let to_fill = to_fill_option.unwrap();

            let potential_fills = self.word_cache.words(to_fill.clone(), self.index);

            let orthogonal_words =
                get_orthogonal_words(&to_fill.entry_location, &entry_location_lookup);

            for potential_fill in potential_fills {
                let new_candidate = fill_one_word(&candidate, &to_fill.clone(), &potential_fill);

                let (valid, tmp) = is_valid_grid(
                    &new_candidate,
                    &orthogonal_words,
                    self.index,
                    used_words,
                    &mut self.is_valid_cache,
                );
                used_words = tmp;
                used_words.clear();

                if valid {
                    if entries_left == 1 || !new_candidate.contents.contains(' ') {
                        return Ok(new_candidate);
                    }
                    candidates.push(new_candidate);
                }
            }
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
                            Direction::Down,
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

#[cfg(test)]
mod tests {

    use crate::{crossword::Direction, fill::Fill, index::Index};

    use crate::Crossword;

    use std::{cmp::Ordering, time::Instant};

    use super::{EntryLocationToFill, Filler};

    #[test]
    fn medium_grid() {
        let grid = Crossword::from_string(
            String::from(
                "
STRAWBERRY*    
          *    
          *    
   *    **     
***   **       
**         *   
*         *    
     *   *     
    *         *
   *         **
       **   ***
     **    *   
    *          
    *          
    *          
",
            ),
            15,
            15,
        )
        .unwrap();

        println!("{}", grid);

        let now = Instant::now();
        let index = Index::build_default();
        let mut filler = Filler::new(&index);
        let filled_puz = filler.fill(&grid, None).unwrap();
        println!("Filled in {} seconds.", now.elapsed().as_secs());
        println!("{}", filled_puz);
    }

    #[test]
    fn test_prefilled_grid_invalid_word() {
        let grid = Crossword::from_string(
            String::from(
                "
  B  
     
RENAI
",
            ),
            5,
            3,
        )
        .unwrap();

        println!("{}", grid);

        let now = Instant::now();
        let index = Index::build_default();
        let mut filler = Filler::new(&index);
        let filled_puz = filler.fill(&grid, None).unwrap();
        println!("Filled in {} seconds.", now.elapsed().as_secs());
        println!("{}", filled_puz);
    }

    #[test]
    fn test_weighted_simple() {
        let grid = Crossword::from_string(
            String::from(
                "
     
     
     
",
            ),
            5,
            3,
        )
        .unwrap();

        println!("{}", grid);

        let now = Instant::now();
        let words = vec![
            (String::from("BBBBB"), 2),
            (String::from("AAAAA"), 1),
            (String::from("BBB"), 0),
            (String::from("AAA"), 2),
        ];

        let index = Index::build(words);
        let mut filler = Filler::new(&index);

        let entry_locations_to_fill = vec![EntryLocationToFill {
            start_row: 0,
            start_col: 0,
            direction: Direction::Across,
        }];
        let filled_puz = filler.fill(&grid, Some(&entry_locations_to_fill)).unwrap();
        println!("Filled in {} seconds.", now.elapsed().as_secs());
        println!("{}", filled_puz);
    }

    #[test]
    fn medium_grid_specified_entries() {
        let grid = Crossword::from_string(
            String::from(
                "
STRAWBERRY*    
          *    
          *    
   *    **     
***   **       
**         *   
*         *    
     *   *     
    *         *
   *         **
       **   ***
     **    *   
    *          
    *          
    *          
",
            ),
            15,
            15,
        )
        .unwrap();

        println!("{}", grid);

        let now = Instant::now();
        let index = Index::build_default();
        let mut filler = Filler::new(&index);

        let entry_locations_to_fill = vec![
            EntryLocationToFill {
                start_row: 0,
                start_col: 0,
                direction: Direction::Down,
            },
            EntryLocationToFill {
                start_row: 0,
                start_col: 1,
                direction: Direction::Down,
            },
        ];

        let filled_puz = filler.fill(&grid, Some(&entry_locations_to_fill)).unwrap();
        println!("Filled in {} seconds.", now.elapsed().as_secs());
        println!("{}", filled_puz);
    }
}
