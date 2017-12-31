use state::*;
use fnv::FnvHashMap;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Strategy {
    Attack(ID),
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

    pub fn docking_at(&self, planet: ID) -> i32 {
        self.ships.values()
            .filter_map(|s| if let &Strategy::Dock(id) = s { Some(id) } else { None })
            .filter(|&id| id == planet )
            .count() as i32
    }

    pub fn set(&mut self, ship: ID, strategy: Strategy) {
        self.ships.insert(ship, strategy);
    }

    pub fn clear(&mut self, ship: ID) {
        self.ships.remove(&ship);
    }
}

pub fn closest_planet<'a, 'b, 'c>(
    ship: &'a Ship,
    planets: &'b Planets,
    strategy: &'c Strategies) -> Option<&'b Planet> 
{
    planets.values()
        .filter(|planet| planet.spots > strategy.docking_at(planet.id))
        .min_by_key(|planet| {
            let x = planet.x - ship.x;
            let y = planet.y - ship.y;
            (x*x + y*y) as i32
        })
}
