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

    pub fn execute(s: &mut State) {
        let mut resolved = FnvHashSet::default();
        let (mut ships, enemies): (Vec<_>, Vec<_>) = s.ships
            .values()
            .filter(|ship| !s.docked.contains_key(&ship.id))
            .partition(|ship| ship.owner == s.id);

        ships.sort_unstable_by_key(|ship| {
            let &(_, ref e) = s.scout.get_combat(ship.id);
            -(e.len() as i32)
        });

        // Resolve combat
        info!("Resolving combat...");
        for ship in &ships {
            if resolved.contains(&ship.id) { continue }
            let &(ref allies, ref enemies) = s.scout.get_combat(ship.id);
            let mut a = allies.iter()
                .filter(|ally| !ally.is_docked())
                .cloned()
                .collect::<Vec<_>>();
                a.push((*ship).clone());

            // Midpoint
            let (x, y) = a.iter()
                .map(|ship| (ship.x, ship.y))
                .fold((0.0, 0.0), |(xa, ya), (x, y)| (x + xa, y + ya));
            let (x, y) = (x / (a.len() as f64), y / (a.len() as f64));

            let (d, e): (Vec<_>, Vec<_>) = enemies.into_iter()
                .partition(|enemy| enemy.is_docked());

            if e.len() == 0 && d.len() == 0 { continue }

            if d.len() > 0 && (a.len() >= d.len() + 2 || e.len() < a.len()) {
                let d = d.into_iter()
                    .min_by(|a, b| {
                        ((a.y - y).hypot(a.x - x)).partial_cmp(
                        &(b.y - y).hypot(b.x - x)).unwrap()
                    }).unwrap();
                let mut a = a.into_iter()
                    .filter(|ally| !resolved.contains(&ally.id))
                    .collect::<Vec<_>>();
                a.sort_unstable_by(|a, b| {
                    a.distance_to(&d).partial_cmp(&b.distance_to(&d)).unwrap()
                });
                let a = a.into_iter().take(SQUADRON_SIZE).collect::<Vec<_>>();
                for ally in &a {
                    resolved.insert(ally.id);
                }
                for command in navigate_clump_to_enemy(&mut s.grid, &a, &d) {
                    s.queue.push(&command);
                }
            } else if e.len() >= a.len() {
                resolved.insert(ship.id);
                s.queue.push(&navigate_from_enemies(&mut s.grid, ship, &e));
                for ally in allies {
                    if !resolved.contains(&ally.id) {
                        resolved.insert(ally.id);
                        s.queue.push(&navigate_from_enemies(&mut s.grid, ally, &e));
                    }
                }
            } else if e.len() > 1 && a.len() > e.len() {
                let e = e.into_iter()
                    .min_by(|a, b| {
                        ((a.y - y).hypot(a.x - x)).partial_cmp(
                        &(b.y - y).hypot(b.x - x)).unwrap()
                    }).unwrap();
                let mut a = a.into_iter()
                    .filter(|ally| !resolved.contains(&ally.id))
                    .take(SQUADRON_SIZE)
                    .collect::<Vec<_>>();
                a.sort_unstable_by(|a, b| {
                    a.distance_to(&e).partial_cmp(&b.distance_to(&e)).unwrap()
                });
                let a = a.into_iter().take(SQUADRON_SIZE).collect::<Vec<_>>();
                for ally in &a {
                    resolved.insert(ally.id);
                }
                for command in navigate_clump_to_enemy(&mut s.grid, &a, &e) {
                    s.queue.push(&command);
                }
            }
        }

        // Resolve attacking
        info!("Resolving attacking...");
        for (enemy, allies) in s.tactics.attack.iter() {
            let enemy = &s.ships[enemy];
            for ally in allies {
                if resolved.contains(&ally) { continue }
                let ship = &s.ships[ally];
                resolved.insert(ship.id);
                s.queue.push(&navigate_to_enemy(&mut s.grid, ship, enemy));
            }
        }

        // Resolve docking
        info!("Resolving docking...");
        for (planet, allies) in s.tactics.dock.iter() {
            let planet = &s.planets[planet];
            for ship in allies {
                if resolved.contains(ship) || !s.ships.contains_key(ship) { continue }
                let ship = &s.ships[ship];

                resolved.insert(ship.id);
                if ship.in_docking_range(planet) {
                    s.docked.insert(ship.id, planet.id);
                    s.queue.push(&dock(&ship, &planet));
                } else {
                    s.queue.push(&navigate_to_planet(&mut s.grid, &ship, &planet))
                }
            }
        }

        // Resolve defense
        info!("Resolving defense...");
        for (planet, allies) in s.tactics.defend.iter() {
            let planet = &s.planets[planet];
            let &(_, ref e) = s.scout.get_env(planet.id);
            let enemy = e.iter()
                .min_by(|a, b| {
                    a.distance_to(&planet).partial_cmp(
                    &b.distance_to(&planet)).unwrap()
                }).expect("Defend called on non-contested planet");
            for ship in allies {
                if resolved.contains(ship) { continue }
                let ship = &s.ships[ship];
                resolved.insert(ship.id);
                s.queue.push(&navigate_to_enemy(&mut s.grid, &ship, &enemy))
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
                    });

                if let Some(enemy) = enemy {
                    let ally = ships.iter()
                        .filter(|ally| {
                            let &(ref a, _) = s.scout.get_combat(ally.id);
                            a.len() < 8
                        })
                        .min_by(|&a, &b| {
                            enemy.distance_to(a).partial_cmp(
                            &enemy.distance_to(b)).unwrap()
                        });

                    if let Some(ally) = ally {
                        resolved.insert(ship.id);
                        s.queue.push(&navigate_to_ally(&mut s.grid, ship, ally));
                    }
                }
            }
        }

        s.queue.flush();
    }
}
