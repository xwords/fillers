use rand::prelude::*;
use rustc_hash::{FxHashMap, FxHasher};
use std::hash::{Hash, Hasher};

use crate::index::Index;

#[derive(Clone)]
pub struct CachedWords {
    words_cache: FxHashMap<u64, Vec<String>>,
}

impl CachedWords {
    pub fn default() -> CachedWords {
        CachedWords {
            words_cache: FxHashMap::default(),
        }
    }

    pub fn words<T: Iterator<Item = char> + Clone>(
        &mut self,
        pattern: T,
        index: &Index,
    ) -> &Vec<String> {
        let mut hasher = FxHasher::default();
        for c in pattern.clone() {
            c.hash(&mut hasher);
        }
        let key = hasher.finish();

        self.words_cache
            .entry(key)
            .or_insert_with(|| index.words(pattern))
    }
}

#[derive(Clone)]
pub struct CachedIsValid {
    is_valid_cache: FxHashMap<u64, bool>,
}

impl CachedIsValid {
    pub fn default() -> CachedIsValid {
        CachedIsValid {
            is_valid_cache: FxHashMap::default(),
        }
    }

    pub fn is_valid<T: Iterator<Item = char> + Clone>(&mut self, chars: T, index: &Index) -> bool {
        let mut hasher = FxHasher::default();
        for c in chars.clone() {
            c.hash(&mut hasher);
        }
        let key = hasher.finish();

        *self
            .is_valid_cache
            .entry(key)
            .or_insert_with(|| index.is_valid(chars))
    }
}
