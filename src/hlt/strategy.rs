use fnv::FnvHashMap;
use hlt::state::*;
use hlt::command::*;
use hlt::collision::*;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Strategy {
    Aggressive,
    Neutral,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Tactic {
    Attack(ID),
    Dock(ID),
    Travel(ID),
    Harass(ID),
}

pub struct Plan {
    pub strategy: Strategy,
    ships: FnvHashMap<ID, Tactic>,
    attack: FnvHashMap<ID, Vec<ID>>,
    dock: FnvHashMap<ID, Vec<ID>>,
    travel: FnvHashMap<ID, Vec<ID>>,
    harass: FnvHashMap<ID, ID>,
}

impl Plan {
    pub fn new() -> Self {
        Plan {
            strategy: Strategy::Neutral,
            ships: FnvHashMap::default(),
            attack: FnvHashMap::default(),
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
                    .swap_remove(ship);
            },
            Tactic::Dock(planet) => {
                self.dock.get_mut(&planet)
                    .unwrap()
                    .swap_remove(ship);
            },
            Tactic::Travel(planet) => {
                self.travel.get_mut(&planet)
                    .unwrap()
                    .swap_remove(ship);
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

    pub fn docking_at(&self, planet: ID) -> i32 {
        Self::count(&self.dock, planet) +
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

    pub fn execute(s: &mut State) {
        for (target, allies) in s.plan.attack.iter() {
            let target = &s.ships[&target];
            let mut allies = allies.iter()
                .map(|ally| &s.ships[&ally])
                .cloned()
                .collect::<Vec<_>>();
            allies.sort_unstable_by(|a, b| {
                a.distance_to(&target).partial_cmp(
                &b.distance_to(&target)).unwrap()
            });
            for ally in allies {
                s.queue.push(&navigate_to_enemy(&mut s.grid, &ally, target));
            }
        }

        for (planet, allies) in s.plan.dock.iter() {
            let planet = &s.planets[&planet];
            let allies = allies.iter()
                .map(|ally| &s.ships[&ally])
                .cloned()
                .collect::<Vec<_>>();
            for ally in &allies {
                s.queue.push(&dock(ally, planet));
            }
        }

        for (planet, allies) in s.plan.travel.iter() {
            let planet = &s.planets[&planet];
            let mut allies = allies.iter()
                .map(|ally| &s.ships[&ally])
                .cloned()
                .collect::<Vec<_>>();
            allies.sort_unstable_by(|a, b| {
                a.distance_to(&planet).partial_cmp(
                &b.distance_to(&planet)).unwrap()
            });
            for ally in allies {
                s.queue.push(&navigate_to_planet(&mut s.grid, &ally, planet));
            }
        }

        for (&planet, &ally) in s.plan.harass.iter() {
            let ship = &s.ships[&ally];
            let planet = &s.planets[&planet];
            let (docked, threats): (Vec<_>, Vec<_>) = s.grid
                .near_enemies(&ship, 14.0, &s.ships)
                .into_iter()
                .partition(|enemy| enemy.is_docked());

            if threats.len() > 0 {
                let avoid = s.grid.near_allies(&ship, 35.0, &s.ships)
                    .into_iter()
                    .chain(threats.into_iter())
                    .collect::<Vec<_>>();
                s.queue.push(
                    &navigate_to_distract(&mut s.grid, &ship, &avoid)
                );
            } else {
                let docked = planet.ships.iter()
                    .map(|enemy| s.ships[&enemy].clone())
                    .min_by(|a, b| {
                        ship.distance_to(&a).partial_cmp(
                        &ship.distance_to(&b)).unwrap()
                    }).unwrap();
                s.queue.push(
                    &navigate_to_enemy(&mut s.grid, &ship, &docked)
                );
            }
        }
    }
}
