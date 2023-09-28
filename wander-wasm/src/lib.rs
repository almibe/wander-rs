// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! This project exposes functionality from the Rust implementation of Wander to WASM and JS runtimes thanks to wasm-bindgen and wasm-pack.

mod utils;

use wander::{NoHostType, WanderError, WanderValue};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn run(script: String) -> JsValue {
    let mut bindings = wander::preludes::common::<NoHostType>();
    match wander::run(&script, &mut bindings) {
        Ok(value) => serde_wasm_bindgen::to_value(&value).unwrap(),
        Err(err) => {
            serde_wasm_bindgen::to_value(&Err::<WanderValue<NoHostType>, WanderError>(err)).unwrap()
        }
    }
}
