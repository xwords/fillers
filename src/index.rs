use serde::{Deserialize, Serialize};
use std::{fs::File, io::BufRead, io::BufReader, path::Path};

use rustc_hash::FxHashMap;

#[derive(Clone, Deserialize, Serialize)]
pub struct TrieNode {
    contents: Option<char>,
    children: FxHashMap<char, TrieNode>,
    terminal: bool,
}

impl TrieNode {
    fn add(mut self, chars: &str) -> TrieNode {
        match chars.as_bytes().get(0) {
            Some(c) => match self.children.remove_entry(&(*c as char)) {
                Some((_, child)) => {
                    self.children.insert(*c as char, child.add(&chars[1..]));
                }
                None => {
                    let new_child = TrieNode {
                        contents: Some(*c as char),
                        children: FxHashMap::default(),
                        terminal: false,
                    };
                    self.children.insert(*c as char, new_child.add(&chars[1..]));
                }
            },
            None => {
                self.terminal = true;
            }
        }
        self
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
    pub fn build(words: Vec<String>) -> Index {
        let mut trie_root = TrieNode {
            contents: None,
            children: FxHashMap::default(),
            terminal: false,
        };

        for word in words.iter() {
            trie_root = trie_root.add(word);
        }

        Index { trie_root }
    }

    fn is_valid<T: Iterator<Item = char> + Clone>(&self, chars: T) -> bool {
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
        let lines = lines_from_file("./WL-SP.txt")
            .into_iter()
            .filter(|s| s.len() > 2)
            .collect();

        let index = Index::build(lines);
    }
}