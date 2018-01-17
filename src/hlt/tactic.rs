use fnv::{FnvHashMap, FnvHashSet};
use hlt::state::*;
use hlt::command::*;
use hlt::constants::*;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Tactic {
    Attack(ID),
    Raid(ID),
    Dock(ID),
    Defend(ID),
}

pub struct Tactics {
    ships: FnvHashMap<ID, Tactic>,
    attack: FnvHashMap<ID, Vec<ID>>,
    raid: FnvHashMap<ID, Vec<ID>>,
    defend: FnvHashMap<ID, Vec<ID>>,
    dock: FnvHashMap<ID, Vec<ID>>,
}

impl Tactics {
    pub fn new() -> Self {
        Tactics {
            ships: FnvHashMap::default(),
            attack: FnvHashMap::default(),
            raid: FnvHashMap::default(),
            defend: FnvHashMap::default(),
            dock: FnvHashMap::default(),
        }
    }

    pub fn set(&mut self, ship: ID, tactic: Tactic) {
        if let Some(previous) = self.ships.insert(ship, tactic) {
            self.remove(ship, previous);
        }
        match tactic {
            Tactic::Attack(enemy) => {
                self.attack.entry(enemy).or_insert(Vec::new()).push(ship);
            },
            Tactic::Raid(planet) => {
                self.raid.entry(planet).or_insert(Vec::new()).push(ship);
            },
            Tactic::Defend(planet) => {
                self.defend.entry(planet).or_insert(Vec::new()).push(ship);
            },
            Tactic::Dock(planet) => {
                self.dock.entry(planet).or_insert(Vec::new()).push(ship);
            },
        }
    }

    fn remove(&mut self, ship: ID, tactic: Tactic) {
        match tactic {
            Tactic::Attack(enemy) => {
                self.attack.get_mut(&enemy)
                    .unwrap()
                    .retain(|&id| id != ship);
            },
            Tactic::Raid(planet) => {
                self.raid.get_mut(&planet)
                    .unwrap()
                    .retain(|&id| id != ship);
            },
            Tactic::Defend(planet) => {
                self.defend.get_mut(&planet)
                    .unwrap()
                    .retain(|&id| id != ship);
            },
            Tactic::Dock(planet) => {
                self.dock.get_mut(&planet)
                    .unwrap()
                    .retain(|&id| id != ship);
            },
        }
    }

    fn count(map: &FnvHashMap<ID, Vec<ID>>, id: ID) -> usize {
        match map.get(&id) {
            None => 0,
            Some(list) => list.len(),
        }
    }

    pub fn is_attacking(&self, ship: ID) -> bool {
        if let Some(&Tactic::Attack(_)) = self.ships.get(&ship) {
            true
        } else { false }
    }

    pub fn raiding(&self, planet: ID) -> usize {
        Self::count(&self.raid, planet)
    }

    pub fn defending(&self, planet: ID) -> usize {
        Self::count(&self.defend, planet)
    }

    pub fn docking(&self, planet: ID) -> usize {
        Self::count(&self.dock, planet)
    }
}
