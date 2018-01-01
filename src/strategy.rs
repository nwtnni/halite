use state::*;
use std::i32::MAX;
use constants::MAX_GROUP_SIZE;
use fnv::FnvHashMap;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Strategy {
    Attack(ID),
    Assemble(ID),
    Follow(ID),
    Dock(ID),
}

#[derive(Default)]
pub struct Strategies {
    ships: FnvHashMap<ID, Strategy>,
}

impl Strategies {
    pub fn new() -> Self {
        Strategies { ships: FnvHashMap::default() }
    }

    pub fn get(&self, ship: ID) -> Option<Strategy> {
        self.ships.get(&ship).cloned()
    }

    pub fn docking_at(&self, planet: ID, planets: &Planets) -> i32 {
        self.ships.values()
            .filter_map(|s| if let &Strategy::Dock(id) = s { Some(id) } else { None })
            .filter(|&id| id == planet && planets.contains_key(&id))
            .count() as i32
    }

    pub fn assembling(&self, ship: ID, ships: &Ships) -> i32 {
        self.ships.values()
            .filter_map(|s| if let &Strategy::Assemble(id) = s { Some(id) } else { None })
            .filter(|&id| id == ship && ships.contains_key(&id))
            .count() as i32
    }

    pub fn following(&self, ship: ID, ships: &Ships) -> i32 {
        self.ships.values()
            .filter_map(|s| {
                if let &Strategy::Follow(id) = s { Some(id) } else {
                    if let &Strategy::Assemble(id) = s { Some(id) } else {
                        None
                    }
                }
            })
            .filter(|&id| id == ship && ships.contains_key(&id))
            .count() as i32

    }

    pub fn attacking(&self, ship: ID, ships: &Ships) -> i32 {
        self.ships.values()
            .filter_map(|s| if let &Strategy::Attack(id) = s { Some(id) } else { None })
            .filter(|&id| id == ship && ships.contains_key(&id))
            .count() as i32
    }

    pub fn attack_group(&self, ship: &Ship, ships: &Ships) -> Option<ID> {
        self.ships.iter()
            .filter_map(|(&id, s)| if let &Strategy::Attack(_) = s { Some(id) } else { None })
            .filter(|&id| self.following(id, ships) < MAX_GROUP_SIZE && id != ship.id)
            .min_by_key(|id| {
                match ships.get(id) {
                    None => MAX,
                    Some(target) => (target.y - ship.y).hypot(target.x - ship.x) as i32,
                }
            }).and_then(|id| {
                if ships.contains_key(&id) { Some(id) } else { None }
            })
    }

    pub fn set(&mut self, ship: ID, strategy: Strategy) {
        self.ships.insert(ship, strategy);
    }

    pub fn clear(&mut self, ship: ID) {
        self.ships.remove(&ship);
    }

    pub fn clean(&mut self, ships: &mut Ships) {
        self.ships.retain(|id, _| {
            ships.contains_key(id)
        })
    }
}

pub fn best_planet<'a, 'b, 'c>(
    ship: &'a Ship,
    planets: &'b Planets,
    strategy: &'c Strategies) -> Option<&'b Planet>
{
    planets.values()
        .filter(|planet| planet.spots > strategy.docking_at(planet.id, planets))
        .min_by_key(|planet| {
            let d = (planet.y - ship.y).hypot(planet.x - ship.x);
            let v = planet.value();
            (d - v) as i32
        })
}

pub fn best_target<'a, 'b>(
    ship: &'a Ship,
    ships: &'a Ships) -> Option<&'a Ship>
{
    ships.values()
        .filter(|other| other.owner != ship.owner)
        .min_by_key(|other| {
            let d = (other.y - ship.y).hypot(other.x - ship.x);
            let v = other.value();
            (d - v) as i32
        })
}
