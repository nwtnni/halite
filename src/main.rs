#![allow(non_snake_case)]
#[macro_use]
extern crate log;
extern crate fnv;
extern crate simplelog;
mod hlt;

use simplelog::*;
use std::fs::File;
use hlt::state::State;
use hlt::general::General;

fn main() {
    WriteLogger::init(LogLevelFilter::Info, Config::default(), File::create("hlt.log").unwrap()).unwrap();
    let state = State::new();
    State::send_ready("nwtnni");
    state.run()
} 
