use fnv::FnvHashMap;
use hlt::state::*;

pub struct Scout {
    ships: FnvHashMap<ID, Vec<Ship>>,
    planets: FnvHashMap<ID, Vec<Planet>>,
}

impl Scout {
    pub fn new(ships: &Ships, planets: &Planets) -> Self {
        let mut ordered_ships = FnvHashMap::default();
        let mut ordered_planets = FnvHashMap::default();
        for ship in ships.values() {
            let mut s = ships.values()
                .cloned()
                .collect::<Vec<_>>();
            s.sort_unstable_by(|a, b| {
                ship.distance_to(&a).partial_cmp(
                &ship.distance_to(&b)).unwrap()
            });
            s.remove(0);
            ordered_ships.insert(ship.id, s);

            let mut p = planets.values()
                .cloned()
                .collect::<Vec<_>>();
            p.sort_unstable_by(|a, b| {
                ship.distance_to(&a).partial_cmp(
                &ship.distance_to(&b)).unwrap()
            });
            ordered_planets.insert(ship.id, p);
        }
        Scout { ships: ordered_ships, planets: ordered_planets }
    }
}
