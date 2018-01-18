use fnv::FnvHashMap;
use std::cmp::*;
use hlt::state::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tactic {
    Aid(Ship, Ship),
    Attack(Ship, Ship),
    Dock(Ship, Planet),
}

pub struct Tactics {
    aiding: FnvHashMap<Ship, Vec<Ship>>,
    attacking: FnvHashMap<Ship, Vec<Ship>>,
    docking: FnvHashMap<Planet, Vec<Ship>>,
}

impl Tactic {
    fn distance(&self) -> f64 {
        match self {
            &Tactic::Aid(ship, ally) => (ally.y - ship.y).hypot(ally.x - ship.x),
            &Tactic::Attack(ship, enemy) => (enemy.y - ship.y).hypot(enemy.x - ship.x),
            &Tactic::Dock(ship, planet) => (planet.y - ship.y).hypot(planet.x - ship.x),
        }
    }
}

impl PartialOrd for Tactic {
    fn partial_cmp(&self, other: &Tactic) -> Option<Ordering> {
        self.distance().partial_cmp(&other.distance())
    }
}

impl Ord for Tactic {
    fn cmp(&self, other: &Tactic) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}
