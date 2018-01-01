#![allow(non_snake_case)]
extern crate fnv;
mod hlt;

use hlt::state::State;
use hlt::general::General;

fn main() {
    State::new().run()
} 
