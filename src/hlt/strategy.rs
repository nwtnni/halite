use fnv::FnvHashMap;
use hlt::state::*;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Tactic {
    Attack(ID),
    Defend(ID),
    Dock(ID),
    Retreat(ID),
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

    pub fn docking_at(&self, planet: ID) -> i32 {
        self.ships.values()
            .filter(|&t| {
                if let &Tactic::Dock(id) = t { planet == id } else { false }
            })
            .count() as i32
    }

    pub fn attacking(&self, ship: ID) -> i32 {
        self.ships.values()
            .filter(|&t| {
                if let &Tactic::Attack(id) = t { ship == id } else { false }
            })
            .count() as i32
    }

    pub fn defending(&self, planet: ID) -> i32 {
        self.ships.values()
            .filter(|&t| {
                if let &Tactic::Defend(id) = t { planet == id } else { false }
            })
            .count() as i32
    }

    pub fn is_available(&self, ship: ID) -> bool {
        self.ships.get(&ship) == None
    }

    pub fn has_target(&self, ship: ID) -> Option<ID> {
        if let Some(&Tactic::Attack(id)) = self.ships.get(&ship) { Some(id) }
        else { None }
    }
}
