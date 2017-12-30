use state::ID;
use fnv::FnvHashMap;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Strategy {
    Dock(ID),
}

#[derive(Default)]
pub struct Strategies {
    ships: FnvHashMap<ID, Strategy>,
}

impl Strategies {
    pub fn new() -> Self {
        Strategies { ships: FnvHashMap::default() }
    }

    pub fn get(&self, ship: ID) -> Option<Strategy> {
        self.ships.get(&ship).cloned()
    }

    pub fn docking_at(&self, planet: ID) -> i32 {
        self.ships.values()
            .filter(|&&Strategy::Dock(id)| id == planet )
            .count() as i32
    }

    pub fn set(&mut self, ship: ID, strategy: Strategy) {
        self.ships.insert(ship, strategy);
    }

    pub fn clear(&mut self, ship: ID) {
        self.ships.remove(&ship);
    }
}
