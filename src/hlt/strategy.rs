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
}

pub struct Plan {
    pub strategy: Strategy,
    ships: FnvHashMap<ID, Tactic>,
    attack: FnvHashMap<ID, Vec<ID>>,
    dock: FnvHashMap<ID, Vec<ID>>,
    travel: FnvHashMap<ID, Vec<ID>>,
}

impl Plan {
    pub fn new() -> Self {
        Plan { 
            strategy: Strategy::Neutral,
            ships: FnvHashMap::default(),
            attack: FnvHashMap::default(),
            dock: FnvHashMap::default(),
            travel: FnvHashMap::default(),
        }
    }

    pub fn set(&mut self, ship: ID, tactic: Tactic) {
        if let Some(previous) = self.ships.insert(ship, tactic) {
            match previous {
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
            }
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
        }
    }

    pub fn clear(&mut self) {
        self.ships.clear();
        self.attack.clear();
        self.travel.clear();
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

    pub fn has_target(&self, ship: ID) -> Option<ID> {
        if let Some(&Tactic::Attack(id)) = self.ships.get(&ship) { Some(id) }
        else { None }
    }

    pub fn has_planet(&self, ship: ID) -> Option<ID> {
        if let Some(&Tactic::Dock(id)) = self.ships.get(&ship) { Some(id) }
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
            for ally in allies {
                s.queue.push(&dock(&ally, planet));
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
    }
}
