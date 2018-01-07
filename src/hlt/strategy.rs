use fnv::{FnvHashMap, FnvHashSet};
use hlt::state::*;
use hlt::command::*;
use hlt::constants::*;
use hlt::collision::*;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Tactic {
    Attack(ID),
    Defend(ID),
    Dock(ID),
    Travel(ID),
}

pub struct Plan {
    ships: FnvHashMap<ID, Tactic>,
    attack: FnvHashMap<ID, Vec<ID>>,
    defend: FnvHashMap<ID, Vec<ID>>,
    dock: FnvHashMap<ID, Vec<ID>>,
    travel: FnvHashMap<ID, Vec<ID>>,
}

impl Plan {
    pub fn new() -> Self {
        Plan {
            ships: FnvHashMap::default(),
            attack: FnvHashMap::default(),
            defend: FnvHashMap::default(),
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
            Tactic::Defend(planet) => {
                self.defend.entry(planet).or_insert(Vec::new()).push(ship);
            },
            Tactic::Dock(planet) => {
                self.dock.entry(planet).or_insert(Vec::new()).push(ship);
            },
            Tactic::Travel(planet) => {
                self.travel.entry(planet).or_insert(Vec::new()).push(ship);
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
        }
    }

    fn count(map: &FnvHashMap<ID, Vec<ID>>, id: ID) -> i32 {
        match map.get(&id) {
            None => 0,
            Some(list) => list.len() as i32,
        }
    }

    pub fn attacking(&self, planet: ID) -> i32 {
        Self::count(&self.attack, planet)
    }

    pub fn defending(&self, planet: ID) -> i32 {
        Self::count(&self.defend, planet)
    }

    pub fn docking_at(&self, planet: ID) -> i32 {
        Self::count(&self.dock, planet) +
        Self::count(&self.travel, planet)
    }

    pub fn docked_at(&self, planet: ID) -> i32 {
        Self::count(&self.dock, planet)
    }

    pub fn traveling_to(&self, planet: ID) -> i32 {
        Self::count(&self.travel, planet)
    }

    pub fn is_available(&self, ship: ID) -> bool {
        self.ships.get(&ship) == None
    }

    pub fn is_attacking(&self, ship: ID) -> bool {
        if let Some(&Tactic::Attack(_)) = self.ships.get(&ship) {
            true
        } else { false }
    }

    pub fn execute(s: &mut State) {
        let mut resolved: FnvHashSet<ID> = FnvHashSet::default();
        let player = &s.players[s.id];
        let ships = &player.ships.iter()
            .filter(|ship| !s.docked.contains_key(ship))
            .map(|ship| s.ships[&ship].clone())
            .collect::<Vec<_>>();

        info!("Resolving combat");
        for ship in ships {
            if resolved.contains(&ship.id) { continue }
            let (d, e): (Vec<_>, Vec<_>) = s.grid
                .near_enemies(&ship, COMBAT_RADIUS, &s.ships)
                .into_iter()
                .partition(|enemy| enemy.is_docked());

            if d.len() == 0 && e.len() == 0 { continue }

            let mut a = s.grid.near_allies(&ship, COMBAT_RADIUS, &s.ships)
                .into_iter()
                .filter(|ally| !ally.is_docked() && !resolved.contains(&ally.id))
                .cloned()
                .collect::<Vec<_>>();
                a.push(ship.clone());

            if a.len() < e.len() + 1 { continue }

            let a = a.into_iter().take(e.len() + 1).collect::<Vec<_>>();
            for ship in &a { resolved.insert(ship.id); }

            if d.len() > 0 {
                for ship in &a { resolved.insert(ship.id); }
                for c in navigate_clump_to_enemy(&mut s.grid, a, d[0]) {
                    s.queue.push(&c);
                }
            } else {
                for c in navigate_clump_to_enemy(&mut s.grid, a, e[0]) {
                    s.queue.push(&c);
                }
            }
        }

        info!("Resolving docking");
        for ship in ships {
            if resolved.contains(&ship.id) { continue }

            let candidate = s.grid.near_planets(&ship, 5.0, &s.planets)
                .into_iter()
                .filter(|planet| {
                    ship.in_docking_range(planet) &&
                    !planet.is_enemy(s.id) && 
                    planet.spots > s.plan.docked_at(planet.id)
                })
                .filter(|planet| {
                    s.grid.near_allies(planet, 14.0, &s.ships)
                        .into_iter()
                        .count()
                    >=
                    s.grid.near_enemies(planet, 49.0, &s.ships)
                        .into_iter()
                        .count()
                }).min_by_key(|planet| ship.distance_to(planet) as i32);

            if let Some(planet) = candidate {
                resolved.insert(ship.id);
                s.docked.insert(ship.id, planet.id);
                s.plan.set(ship.id, Tactic::Dock(planet.id));
                s.queue.push(&dock(ship, planet));
            }
        }


        info!("Resolving attacks");
        for (target, allies) in s.plan.attack.iter() {
            let planet = &s.planets[&target];
            let mut allies = allies.into_iter()
                .filter(|ally| !resolved.contains(ally))
                .map(|ally| s.ships[&ally].clone())
                .collect::<Vec<_>>();

            if allies.len() == 0 { continue }

            let (d, e): (Vec<_>, Vec<_>) = s.grid
                .near_enemies(&planet, planet.rad + SCAN_RADIUS, &s.ships)
                .into_iter()
                .partition(|enemy| enemy.is_docked());

            if d.len() == 0 && e.len() == 0 { panic!("Attack called on empty planet") }
            let target = if d.len() > 0 { d[0] } else { e[0] };

            allies.sort_unstable_by(|a, b| {
                a.distance_to(&target).partial_cmp(
                &b.distance_to(&target)).unwrap()
            });

            for a in &allies { resolved.insert(a.id); }
            for c in navigate_clump_to_enemy(&mut s.grid, allies, &target) {
                s.queue.push(&c);
            }
        }

        info!("Resolving defense");
        for (planet, allies) in s.plan.defend.iter() {
            let planet = &s.planets[&planet];
            let allies = allies.iter()
                .filter(|ally| !resolved.contains(ally))
                .map(|ally| &s.ships[&ally])
                .cloned()
                .collect::<Vec<_>>();

            if allies.len() == 0 { continue }

            for ally in &allies {
                let enemy = s.grid.near_enemies(
                    &planet, SCAN_RADIUS + planet.rad, &s.ships
                ).into_iter()
                .min_by(|a, b| {
                    a.distance_to(&ally).partial_cmp(
                    &b.distance_to(&ally)).unwrap()
                }).expect("Defend called with no enemies near");

                resolved.insert(ally.id);
                s.queue.push(
                    &navigate_to_enemy(&mut s.grid, &ally, &enemy)
                );
            }
        }

        info!("Resolving travel");
        for (planet, allies) in s.plan.travel.iter() {
            info!("{:#?}", allies);
            let planet = &s.planets[&planet];
            let mut allies = allies.iter()
                .filter(|ally| !resolved.contains(ally))
                .map(|ally| &s.ships[&ally])
                .cloned()
                .collect::<Vec<_>>();

            if allies.len() == 0 { continue }

            allies.sort_unstable_by(|a, b| {
                a.distance_to(&planet).partial_cmp(
                &b.distance_to(&planet)).unwrap()
            });

            for ally in allies {
                if !ally.in_docking_range(planet) {
                    resolved.insert(ally.id);
                    s.queue.push(&navigate_to_planet(&mut s.grid, &ally, planet));
                }
            }
        }

        info!("Resolving remaining ships");
        let ships = &player.ships.iter()
            .filter(|&&ship| {
                !s.docked.contains_key(&ship)
                && !resolved.contains(&ship)
            })
            .map(|ship| s.ships[&ship].clone())
            .collect::<Vec<_>>();

        for ship in ships {
            let (a, e): (Vec<_>, Vec<_>) = s.ships.values()
                .partition(|ship| ship.owner == s.id);

            let enemy = e.iter()
                .filter(|&other| ship.distance_to(other) > ASSEMBLE_RADIUS)
                .filter(|&enemy| { s.grid.near_enemies(enemy, 7.0, &s.ships).len() > 2 })
                .min_by(|&e1, &e2| {
                    ship.distance_to(e1).partial_cmp(&ship.distance_to(e2)).unwrap()
                });

            let enemy = if let Some(ship) = enemy { ship } else { e[0] };

            let ally = a.iter()
                .filter(|&other| ship.distance_to(other) > ASSEMBLE_RADIUS)
                .min_by(|&a1, &a2| {
                    a1.distance_to(&enemy).partial_cmp(&a2.distance_to(&enemy)).unwrap()
                });

            let ally = if let Some(ship) = ally { ship }
            else {
                a.iter().min_by(|&a1, &a2| {
                    a1.distance_to(&enemy).partial_cmp(&a2.distance_to(&enemy)).unwrap()
                }).expect("No allies")
            };

            s.queue.push(&navigate_to_point(&mut s.grid, ship, (ally.x, ally.y)));
        }
    }
}
