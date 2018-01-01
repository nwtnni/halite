use std::i32::MIN;
use fnv::FnvHashMap;
use hlt::state::*;
use hlt::collision::Grid;
use hlt::constants::{MOB_PENALTY, ENEMY_PENALTY, DOCK_RADIUS, SHIP_RADIUS, SHIP_SPEED};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Strategy {
    Attack(ID),
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

    pub fn attacking(&self, ship: ID, ships: &Ships) -> i32 {
        self.ships.values()
            .filter_map(|s| if let &Strategy::Attack(id) = s { Some(id) } else { None })
            .filter(|&id| id == ship && ships.contains_key(&id))
            .count() as i32
    }

    pub fn set(&mut self, ship: ID, strategy: Strategy) {
        self.ships.insert(ship, strategy);
    }

    pub fn clean(&mut self, ships: &mut Ships) {
        self.ships.retain(|id, _| {
            ships.contains_key(id)
        })
    }
}

pub fn best_planet<'a, 'b, 'c, 'd, 'e>(
    id: ID,
    ship: &'a Ship,
    ships: &'b Ships,
    planets: &'c Planets,
    strategy: &'d Strategies,
    grid: &'e Grid) -> Option<&'c Planet>
{
    planets.values()
        .filter(|&planet| planet.owner.map_or(true, |owner| id != owner))
        .min_by_key(|&planet| {
            let d = (planet.y - ship.y).hypot(planet.x - ship.x);
            let v = planet.value();
            let o = strategy.docking_at(planet.id, planets).pow(2)*MOB_PENALTY;
            let e = grid.near_enemies(planet, ship.id, ships).pow(2)*ENEMY_PENALTY;
            if d < DOCK_RADIUS + SHIP_RADIUS + SHIP_SPEED {
                MIN
            } else {
                ((d - v) as i32) + o + e
            }
        })
}

pub fn best_target<'a, 'b, 'c>(
    ship: &'a Ship,
    ships: &'a Ships,
    strategy: &'b Strategies,
    grid: &'c Grid) -> Option<&'a Ship>
{
    ships.values()
        .filter(|other| other.owner != ship.owner)
        .min_by_key(|&other| {
            let d = (other.y - ship.y).hypot(other.x - ship.x);
            let v = other.value();
            let o = strategy.attacking(other.id, ships)*MOB_PENALTY;
            let e = grid.near_enemies(other, ship.id, ships).pow(2)*ENEMY_PENALTY;
            ((d - v) as i32) + o + e;
        })
}
