use fnv::FnvHashMap;
use hlt::state::*;
use hlt::command::*;
use hlt::constants::*;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Tactic {
    Attack(ID),
    Retreat(ID),
    Dock(ID),
    Defend(ID),
    Travel(ID),
    Harass(ID),
}

pub struct Tactics {
    ships: FnvHashMap<ID, Tactic>,
    attack: FnvHashMap<ID, Vec<ID>>,
    retreat: FnvHashMap<ID, Vec<ID>>,
    defend: FnvHashMap<ID, Vec<ID>>,
    dock: FnvHashMap<ID, Vec<ID>>,
    travel: FnvHashMap<ID, Vec<ID>>,
    harass: FnvHashMap<ID, ID>,
}

impl Tactics {
    pub fn new() -> Self {
        Tactics {
            ships: FnvHashMap::default(),
            attack: FnvHashMap::default(),
            retreat: FnvHashMap::default(),
            defend: FnvHashMap::default(),
            harass: FnvHashMap::default(),
            dock: FnvHashMap::default(),
            travel: FnvHashMap::default(),
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
            Tactic::Retreat(enemy) => {
                self.retreat.entry(enemy).or_insert(Vec::new()).push(ship);
            }
            Tactic::Defend(planet) => {
                self.defend.entry(planet).or_insert(Vec::new()).push(ship);
            },
            Tactic::Dock(planet) => {
                self.dock.entry(planet).or_insert(Vec::new()).push(ship);
            },
            Tactic::Travel(planet) => {
                self.travel.entry(planet).or_insert(Vec::new()).push(ship);
            },
            Tactic::Harass(planet) => {
                self.harass.insert(planet, ship);
            }
        }
    }

    fn remove(&mut self, ship: ID, tactic: Tactic) {
        match tactic {
            Tactic::Attack(enemy) => {
                self.attack.get_mut(&enemy)
                    .unwrap()
                    .retain(|&id| id != ship);
            },
            Tactic::Retreat(enemy) => {
                self.retreat.get_mut(&enemy)
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
            Tactic::Travel(planet) => {
                self.travel.get_mut(&planet)
                    .unwrap()
                    .retain(|&id| id != ship);
            }
            Tactic::Harass(planet) => {
                self.harass.remove(&planet);
            }
        }
    }

    fn count(map: &FnvHashMap<ID, Vec<ID>>, id: ID) -> i32 {
        match map.get(&id) {
            None => 0,
            Some(list) => list.len() as i32,
        }
    }

    pub fn attacking(&self, ship: ID) -> i32 {
        Self::count(&self.attack, ship)
    }

    pub fn defending(&self, planet: ID) -> i32 {
        Self::count(&self.defend, planet)
    }

    pub fn docking_at(&self, planet: ID) -> i32 {
        Self::count(&self.dock, planet) +
        Self::count(&self.travel, planet)
    }

    pub fn traveling_to(&self, planet: ID) -> i32 {
        Self::count(&self.travel, planet)
    }

    pub fn is_available(&self, ship: ID) -> bool {
        self.ships.get(&ship) == None
    }

    pub fn is_victim(&self, planet: ID) -> bool {
        self.harass.get(&planet) != None
    }

    pub fn has_target(&self, ship: ID) -> Option<ID> {
        if let Some(&Tactic::Attack(id)) = self.ships.get(&ship) { Some(id) }
        else { None }
    }

    pub fn execute(_: &mut State) {
    }
}
