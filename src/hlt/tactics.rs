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

    pub fn is_attacking(&self, ship: ID) -> bool {
        if let Some(&Tactic::Attack(_)) = self.ships.get(&ship) {
            true
        } else { false }
    }

    pub fn is_victim(&self, planet: ID) -> bool {
        self.harass.get(&planet) != None
    }

    pub fn has_target(&self, ship: ID) -> Option<ID> {
        if let Some(&Tactic::Attack(id)) = self.ships.get(&ship) { Some(id) }
        else { None }
    }

    pub fn execute(s: &mut State) {
        for (target, allies) in s.tactics.attack.iter() {
            if allies.len() == 0 { continue }
            let target = &s.ships[&target];
            let mut squadrons = allies.into_iter()
                .map(|ally| &s.ships[&ally])
                .cloned()
                .collect::<Vec<_>>();

            squadrons.sort_unstable_by(|a, b| {
                a.distance_to(&target).partial_cmp(
                &b.distance_to(&target)).unwrap()
            });

            for squadron in squadrons.chunks(SQUADRON_SIZE) {
                for command in navigate_clump_to_enemy(&mut s.grid, squadron, &target) {
                    s.queue.push(&command);
                }
            }
        }

        for (enemy, allies) in s.tactics.retreat.iter() {
            if allies.len() == 0 { continue }
            let enemy = &s.ships[&enemy];
            let mut squadrons = allies.iter()
                .map(|ally| &s.ships[&ally])
                .cloned()
                .collect::<Vec<_>>();

            squadrons.sort_unstable_by(|a, b| {
                b.distance_to(&enemy).partial_cmp(
                &a.distance_to(&enemy)).unwrap()
            });

            for squadron in squadrons.chunks(SQUADRON_SIZE) {
                for command in navigate_clump_from_enemy(&mut s.grid, squadron, &enemy) {
                    s.queue.push(&command);
                }
            }
        }

        for (planet, allies) in s.tactics.defend.iter() {
            if allies.len() == 0 { continue }
            let planet = &s.planets[&planet];
            let allies = allies.iter()
                .map(|ally| &s.ships[&ally])
                .cloned()
                .collect::<Vec<_>>();

            for ally in &allies {
                let enemy = s.grid.near_enemies(
                    &planet, defense_radius(&planet), &s.ships
                )
                .into_iter()
                .min_by(|a, b| {
                    a.distance_to(&ally).partial_cmp(
                    &b.distance_to(&ally)).unwrap()
                }).expect("Defend called with no enemies near");

                s.queue.push(
                    &navigate_to_enemy(&mut s.grid, &ally, &enemy)
                );
            }
        }

        for (planet, allies) in s.tactics.dock.iter() {
            if allies.len() == 0 { continue }
            if let &Some(planet) = &s.planets.get(&planet) {
                let allies = allies.iter()
                    .filter_map(|ally| s.ships.get(&ally))
                    .cloned()
                    .collect::<Vec<_>>();
                for ally in &allies {
                    s.queue.push(&dock(ally, planet));
                }
            }
        }

        for (planet, allies) in s.tactics.travel.iter() {
            if allies.len() == 0 { continue }
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

        for (&planet, &ally) in s.tactics.harass.iter() {
            let ship = &s.ships[&ally];
            let planet = &s.planets[&planet];
            let threats = s.grid.near_enemies(&ship, HARASS_RADIUS, &s.ships)
                .into_iter()
                .filter(|enemy| !enemy.is_docked())
                .collect::<Vec<_>>();

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