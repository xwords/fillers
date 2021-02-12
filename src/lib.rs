extern crate js_sys;
extern crate wasm_bindgen;

pub mod crossword;
pub mod fill;
pub mod index;

use crate::crossword::Crossword;

use self::js_sys::Array;
use fill::{EntryLocationToFill, Fill, Filler};
use index::Index;
use js_sys::{JsString, Number};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Solver {
    index: Index,
}

#[wasm_bindgen]
impl Solver {
    pub fn new(words_arr: Array) -> Solver {
        let words: Vec<String> = words_arr.iter().map(|d| d.as_string().unwrap()).collect();

        let index = Index::build(words);
        Solver { index }
    }

    pub fn solve(
        &self,
        grid: JsString,
        rows: Number,
        cols: Number,
        clues_to_fill: JsValue,
    ) -> JsValue {
        let mut filler = Filler::new(&self.index);
        let crossword = Crossword::from_string(
            grid.as_string().unwrap(),
            cols.as_f64().unwrap() as usize,
            rows.as_f64().unwrap() as usize,
        )
        .unwrap();

        match clues_to_fill.is_undefined() {
            true => {
                let candidate = filler.fill(&crossword, None).unwrap();
                candidate.contents.into()
            }
            false => {
                let parsed: Vec<EntryLocationToFill> = clues_to_fill.into_serde().unwrap();
                // return Some(&parsed);
                let candidate = filler.fill(&crossword, Some(&parsed)).unwrap();
                candidate.contents.into()
            }
        }
    }
}
