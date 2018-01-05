#![allow(non_snake_case)]
#[macro_use]
extern crate log;
extern crate fnv;
extern crate simplelog;
mod hlt;

use simplelog::*;
use std::fs::File;
use hlt::state::State;
use hlt::twos;
use hlt::fours;

fn main() {
    WriteLogger::init(LogLevelFilter::Info, Config::default(), File::create("hlt.log").unwrap()).unwrap();
    State::send_ready("nwtnni");
    let mut state = State::new();
    let mut turn = 0;
    if state.players.len() == 2 {
        loop {
            state.update();     
            twos::step(&mut state, turn);
            turn += 1;
        }
    } else {
        loop {
            state.update();     
            fours::step(&mut state, turn);
            turn += 1;
        }
    }
} 
