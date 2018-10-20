#[macro_use]
extern crate log;
extern crate fixedbitset;
extern crate fnv;
extern crate hungarian;

mod constants;
mod command;
mod data;
mod grid;
mod parse;
mod strategy;

pub use data::State;
pub use strategy::execute;
