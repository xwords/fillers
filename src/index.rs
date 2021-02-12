use rand::prelude::SliceRandom;
use serde::{Deserialize, Serialize};
use std::{fs::File, io::BufRead, io::BufReader, path::Path};

use rustc_hash::FxHashMap;

#[derive(Clone, Deserialize, Serialize)]
pub struct TrieNode {
    contents: Option<char>,
    children: FxHashMap<char, TrieNode>,
    terminal: bool,
    weight: Option<i32>,
}

impl TrieNode {
    fn add(mut self, chars: &str, weight: i32) -> TrieNode {
        match chars.as_bytes().get(0) {
            Some(c) => match self.children.remove_entry(&(*c as char)) {
                Some((_, child)) => {
                    self.children
                        .insert(*c as char, child.add(&chars[1..], weight));
                }
                None => {
                    let new_child = TrieNode {
                        contents: Some(*c as char),
                        children: FxHashMap::default(),
                        terminal: false,
                        weight: None,
                    };
                    self.children
                        .insert(*c as char, new_child.add(&chars[1..], weight));
                }
            },
            None => {
                self.terminal = true;
                self.weight = Some(weight);
            }
        }
        self
    }

    fn fill_words<T: Iterator<Item = char> + Clone>(
        &self,
        mut pattern: T,
        partial: &mut String,
        result: &mut Vec<(i32, String)>,
    ) {
        if self.contents.is_some() {
            partial.push(self.contents.unwrap());
        }

        match pattern.next() {
            Some(c) => {
                if c == ' ' {
                    for child in self.children.values() {
                        child.fill_words(pattern.clone(), partial, result);
                    }
                } else {
                    if let Some(child) = self.children.get(&c) {
                        child.fill_words(pattern.clone(), partial, result)
                    }
                }
            }
            None => {
                if self.terminal {
                    result.push((self.weight.unwrap_or(0), partial.clone()));
                }
            }
        }

        if self.contents.is_some() {
            partial.pop();
        }
    }

    fn is_valid<T: Iterator<Item = char> + Clone>(&self, mut chars: T) -> bool {
        match chars.next() {
            None => self.terminal,
            Some(c) => {
                if c == ' ' {
                    for child_node in self.children.values() {
                        if child_node.is_valid(chars.clone()) {
                            return true;
                        }
                    }
                    false
                } else {
                    match self.children.get(&c) {
                        None => false,
                        Some(child_node) => child_node.is_valid(chars.clone()),
                    }
                }
            }
        }
    }
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Index {
    pub trie_root: TrieNode,
}

impl Index {
    pub fn build(words: Vec<(String, i32)>) -> Index {
        let mut trie_root = TrieNode {
            contents: None,
            children: FxHashMap::default(),
            terminal: false,
            weight: None,
        };

        for (word, weight) in words.iter() {
            trie_root = trie_root.add(word, *weight);
        }

        Index { trie_root }
    }

    pub fn build_default() -> Index {
        let lines = lines_from_file("./WL-SP.txt")
            .into_iter()
            .filter(|s| s.len() > 2)
            .map(|w| (w, 0))
            .collect();

        Index::build(lines)
    }

    pub fn words<T: Iterator<Item = char> + Clone>(&self, pattern: T) -> Vec<String> {
        let mut result = Vec::with_capacity(4);
        let mut partial = String::with_capacity(4);
        self.trie_root
            .fill_words(pattern, &mut partial, &mut result);

        let mut rng = rand::thread_rng();
        result.shuffle(&mut rng);
        result.sort_by_key(|t| -t.0);

        result.into_iter().map(|t| t.1).collect()
    }

    pub fn is_valid<T: Iterator<Item = char> + Clone>(&self, chars: T) -> bool {
        self.trie_root.is_valid(chars)
    }
}

fn lines_from_file(filename: impl AsRef<Path>) -> Vec<String> {
    let file = File::open(filename).expect("no such file");
    let buf = BufReader::new(file);
    buf.lines()
        .map(|l| l.expect("Could not parse line"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{lines_from_file, Index};

    #[test]
    fn build_real_index() {
        let index = Index::build_default();
    }
}
