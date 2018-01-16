#![allow(non_snake_case)]
#[macro_use]
extern crate log;
extern crate fnv;
extern crate simplelog;
mod hlt;

use simplelog::*;
use std::fs::File;
use hlt::state::State;
use hlt::strategy;

fn main() {
    WriteLogger::init(LogLevelFilter::Info, Config::default(), File::create("hlt.log").unwrap()).unwrap();
    State::send_ready("nwtnni");
    let mut state = State::new();
    let mut turn = 0;
    loop {
        state.update();
        turn += 1;
    }
} 
