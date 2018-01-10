use fnv::{FnvHashMap, FnvHashSet};
use hlt::state::*;
use hlt::command::*;
use hlt::constants::*;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Tactic {
    Raid(ID),
    Dock(ID),
    Defend(ID),
}

pub struct Tactics {
    ships: FnvHashMap<ID, Tactic>,
    raid: FnvHashMap<ID, Vec<ID>>,
    defend: FnvHashMap<ID, Vec<ID>>,
    dock: FnvHashMap<ID, Vec<ID>>,
}

impl Tactics {
    pub fn new() -> Self {
        Tactics {
            ships: FnvHashMap::default(),
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
            Tactic::Raid(enemy) => {
                self.raid.entry(enemy).or_insert(Vec::new()).push(ship);
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
            Tactic::Raid(enemy) => {
                self.raid.get_mut(&enemy)
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

    fn count(map: &FnvHashMap<ID, Vec<ID>>, id: ID) -> i32 {
        match map.get(&id) {
            None => 0,
            Some(list) => list.len() as i32,
        }
    }

    pub fn attacking(&self, planet: ID) -> i32 {
        Self::count(&self.raid, planet)
    }

    pub fn defending(&self, planet: ID) -> i32 {
        Self::count(&self.defend, planet)
    }

    pub fn docking_at(&self, planet: ID) -> i32 {
        Self::count(&self.dock, planet)
    }

    pub fn is_available(&self, ship: ID) -> bool {
        self.ships.get(&ship) == None
    }

    pub fn execute(s: &mut State) {
        let mut resolved = FnvHashSet::default();
        let ships = s.ships.values()
            .filter(|ship| ship.owner == s.id)
            .filter(|ship| !s.docked.contains_key(&ship.id))
            .collect::<Vec<_>>();

        info!("1");
        // Resolve combat
        for ship in ships {
            let &(ref allies, ref enemies) = s.scout.get_combat(ship.id);
            if enemies.len() > 0 {
                resolved.insert(ship.id);
                s.queue.push(&navigate_to_enemy(&mut s.grid, &ship, &enemies[0]));
            }
        }

        info!("2");
        // Resolve docking
        for (planet, allies) in s.tactics.dock.iter() {
            let planet = &s.planets[planet];
            for ship in allies {
                if resolved.contains(ship) { continue }
                let ship = &s.ships[ship];
                resolved.insert(ship.id);
                if ship.in_docking_range(planet) {
                    s.queue.push(&dock(&ship, &planet));
                } else {
                    s.queue.push(&navigate_to_planet(&mut s.grid, &ship, &planet))
                }
            }
        }

        info!("3");
        // Resolve raids
        for (planet, allies) in s.tactics.raid.iter() {
            let planet = &s.planets[planet];
            for ship in allies {
                if resolved.contains(ship) { continue }
                let ship = &s.ships[ship];
                resolved.insert(ship.id);
                s.queue.push(&navigate_to_planet(&mut s.grid, &ship, &planet))
            }
        }

        info!("4");
        // Resolve defense
        for (planet, allies) in s.tactics.defend.iter() {
            let planet = &s.planets[planet];
            for ship in allies {
                if resolved.contains(ship) { continue }
                let ship = &s.ships[ship];
                resolved.insert(ship.id);
                s.queue.push(&navigate_to_planet(&mut s.grid, &ship, &planet))
            }
        }

        s.queue.flush();
    }
}