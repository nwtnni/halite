use fnv::FnvHashMap;
use hlt::state::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Tactic {
    Attack(ID),
    Dock(ID),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Strategy {
    Neutral,
}

pub struct Plan {
    pub strategy: Strategy,
    ships: FnvHashMap<ID, Tactic>,
}

impl Plan {
    pub fn new() -> Self {
        Plan { strategy: Strategy::Neutral, ships: FnvHashMap::default() }
    }

    pub fn get(&self, ship: ID) -> Option<Tactic> {
        self.ships.get(&ship).cloned()
    }

    pub fn set(&mut self, ship: ID, tactic: Tactic) {
        self.ships.insert(ship, tactic);
    }

    pub fn docking_at(&self, planet: ID, planets: &Planets) -> i32 {
        self.ships.values()
            .filter_map(|s| if let &Tactic::Dock(id) = s { Some(id) } else { None })
            .filter(|&id| id == planet && planets.contains_key(&id))
            .count() as i32
    }

    pub fn attacking(&self, ship: ID, ships: &Ships) -> i32 {
        self.ships.values()
            .filter_map(|s| if let &Tactic::Attack(id) = s { Some(id) } else { None })
            .filter(|&id| id == ship && ships.contains_key(&id))
            .count() as i32
    }

    pub fn clear(&mut self, ships: &mut Ships) {
        self.ships.clear();
    }
}

// pub fn best_planet<'a, 'b, 'c, 'd, 'e>(
//     id: ID,
//     ship: &'a Ship,
//     ships: &'b Ships,
//     planets: &'c Planets,
//     strategy: &'d Strategies,
//     grid: &'e Grid) -> Option<&'c Planet>
// {
//     planets.values()
//         .filter(|&planet| planet.owner.map_or(true, |owner| id != owner))
//         .min_by_key(|&planet| {
//             let d = (planet.y - ship.y).hypot(planet.x - ship.x);
//             let v = planet.value();
//             let o = strategy.docking_at(planet.id, planets).pow(2)*MOB_PENALTY;
//             let e = grid.near_ships(planet, 5.0)
//                 .len().pow(2) as i32 * ENEMY_PENALTY;
//             if d < DOCK_RADIUS + SHIP_RADIUS + SHIP_SPEED {
//                 MIN
//             } else {
//                 ((d - v) as i32) + o + (e as i32)
//             }
//         })
// }

// pub fn best_target<'a, 'b, 'c>(
//     ship: &'a Ship,
//     ships: &'a Ships,
//     strategy: &'b Strategies,
//     grid: &'c Grid) -> Option<&'a Ship>
// {
//     ships.values()
//         .filter(|other| other.owner != ship.owner)
//         .min_by_key(|&other| {
//             let d = (other.y - ship.y).hypot(other.x - ship.x);
//             let v = other.value();
//             let o = strategy.attacking(other.id, ships)*MOB_PENALTY;
//             let e = grid.near_ships(other, 5.0)
//                 .len().pow(2) as i32 *ENEMY_PENALTY;
//             ((d - v) as i32) + o + e;
//         })
// }
