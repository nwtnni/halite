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

    pub fn raiding(&self, planet: ID) -> usize {
        Self::count(&self.raid, planet)
    }

    pub fn defending(&self, planet: ID) -> usize {
        Self::count(&self.defend, planet)
    }

    pub fn docking(&self, planet: ID) -> usize {
        Self::count(&self.dock, planet)
    }

    pub fn execute(s: &mut State) {
        let mut resolved = FnvHashSet::default();
        let (mut ships, enemies): (Vec<_>, Vec<_>) = s.ships
            .values()
            .filter(|ship| !s.docked.contains_key(&ship.id))
            .partition(|ship| ship.owner == s.id);

        ships.sort_unstable_by_key(|ship| {
            let &(_, ref e) = s.scout.get_combat(ship.id);
            e.len()
        });

        // Resolve combat
        info!("Resolving combat...");
        for ship in &ships {
            let &(ref a, ref e) = s.scout.get_combat(ship.id);
            if e.len() > a.len() {
                resolved.insert(ship.id);
                s.queue.push(&navigate_from_enemies(&mut s.grid, ship, e));
            }
        }

        // Resolve docking
        info!("Resolving docking...");
        for (planet, allies) in s.tactics.dock.iter() {
            let planet = &s.planets[planet];
            for ship in allies {
                if resolved.contains(ship) || !s.ships.contains_key(ship) { continue }
                let ship = &s.ships[ship];
                let &(ref a, ref e) = s.scout.get_env(planet.id);
                resolved.insert(ship.id);
                if ship.in_docking_range(planet) && e.len() < a.len() {
                    info!("Ship {} was in docking range with {} to {}",
                          ship.id, a.len(), e.len());
                    s.docked.insert(ship.id, planet.id);
                    s.queue.push(&dock(&ship, &planet));
                } else {
                    info!("Ship {} was not in docking range with {} to {}",
                          ship.id, a.len(), e.len());
                    s.queue.push(&navigate_to_planet(&mut s.grid, &ship, &planet))
                }
            }
        }

        // Resolve raids
        info!("Resolving raids...");
        for (planet, allies) in s.tactics.raid.iter() {
            let planet = &s.planets[planet];
            let target = &s.ships[&planet.ships[0]];
            for ship in allies {
                if resolved.contains(ship) { continue }
                let ship = &s.ships[ship];
                resolved.insert(ship.id);
                s.queue.push(&navigate_to_enemy(&mut s.grid, &ship, &target))
            }
        }

        // Resolve defense
        info!("Resolving defense...");
        for (planet, allies) in s.tactics.defend.iter() {
            let planet = &s.planets[planet];
            let &(_, ref e) = s.scout.get_env(planet.id);
            let docked = &s.ships[&planet.ships[0]];
            let enemy = &e[0];
            for ship in allies {
                if resolved.contains(ship) { continue }
                let ship = &s.ships[ship];
                resolved.insert(ship.id);
                s.queue.push(&navigate_to_defend(&mut s.grid, &ship, &docked, &enemy))
            }
        }

        // Resolve hotspots
        if enemies.len() > 0 && ships.len() > 1 {
            for ship in &ships {
                if resolved.contains(&ship.id) { continue }

                let enemy = enemies.iter()
                    .filter(|enemy| {
                        let &(ref a , ref e) = s.scout.get_combat(enemy.id);
                        e.len() > a.len() || e.len() > 1
                    }).min_by(|&a, &b| {
                        ship.distance_to(a).partial_cmp(
                        &ship.distance_to(b)).unwrap()
                    }).unwrap_or(
                        enemies.iter().min_by(|&a, &b| {
                            ship.distance_to(a).partial_cmp(
                            &ship.distance_to(b)).unwrap()
                        }).expect("No enemies remaining")
                    );

                let ally = ships.iter()
                    .min_by(|&a, &b| {
                        a.distance_to(enemy).partial_cmp(
                        &b.distance_to(enemy)).unwrap()
                    }).unwrap();
                
                resolved.insert(ship.id);
                s.queue.push(&navigate_to_enemy(&mut s.grid, ship, enemy));
            }
        }


        s.queue.flush();
    }
}
