use state::{ID};
use fnv::{FnvBuildHasher, FnvHashMap};
use std::collections::hash_map::HashMap;

#[derive(Debug, Copy, Clone)]
pub enum Strategy {
    Dock(ID),
}

#[derive(Default)]
pub struct Strategies {
    ships: HashMap<ID, Strategy, FnvBuildHasher>,
}

impl Strategies {
    pub fn new() -> Self {
        Strategies { ships: FnvHashMap::default() }
    }

    pub fn get(&self, ship: ID) -> Option<Strategy> {
        self.ships.get(&ship).cloned()
    }

    pub fn set(&mut self, ship: ID, strategy: Strategy) {
        self.ships.insert(ship, strategy);
    }
}
