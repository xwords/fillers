extern crate js_sys;
extern crate wasm_bindgen;

pub mod crossword;
pub mod fill;
pub mod index;

use crate::crossword::Crossword;

use self::js_sys::Array;
use fill::{Fill, Filler};
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

    pub fn solve(&self, grid: JsString, rows: Number, cols: Number) -> JsValue {
        let mut filler = Filler::new(&self.index);
        let crossword = Crossword::from_string(
            grid.as_string().unwrap(),
            cols.as_f64().unwrap() as usize,
            rows.as_f64().unwrap() as usize,
        )
        .unwrap();

        let candidate = filler.fill(&crossword).unwrap();

        candidate.contents.into()
    }
}
