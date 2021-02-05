extern crate js_sys;
extern crate wasm_bindgen;

pub mod crossword;
pub mod fill;

use crate::crossword::Crossword;

use self::js_sys::Array;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Solver {}

#[wasm_bindgen]
impl Solver {
    pub fn new(words_arr: Array) -> Solver {
        Solver {}
    }

    pub fn solve(&self, spec_arr: Array) -> JsValue {
        // let height = spec_arr.into();

        JsValue::NULL
    }
}
