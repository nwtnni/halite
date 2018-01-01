use fnv::FnvHashMap;
use hlt::state::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Tactic {
    Attack(ID),
    Dock(ID),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Strategy {
    Neutral,
}

pub struct Plan {
    pub strategy: Strategy,
    ships: FnvHashMap<ID, Tactic>,
}

impl Plan {
    pub fn new() -> Self {
        Plan { strategy: Strategy::Neutral, ships: FnvHashMap::default() }
    }

    pub fn get(&self, ship: ID) -> Option<Tactic> {
        self.ships.get(&ship).cloned()
    }

    pub fn set(&mut self, ship: ID, tactic: Tactic) {
        self.ships.insert(ship, tactic);
    }

    pub fn docking_at(&self, planet: ID, planets: &Planets) -> i32 {
        self.ships.values()
            .filter_map(|s| if let &Tactic::Dock(id) = s { Some(id) } else { None })
            .filter(|&id| id == planet && planets.contains_key(&id))
            .count() as i32
    }

    pub fn attacking(&self, ship: ID, ships: &Ships) -> i32 {
        self.ships.values()
            .filter_map(|s| if let &Tactic::Attack(id) = s { Some(id) } else { None })
            .filter(|&id| id == ship && ships.contains_key(&id))
            .count() as i32
    }

    pub fn clear(&mut self, ships: &mut Ships) {
        self.ships.clear();
    }
}
