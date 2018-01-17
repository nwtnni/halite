use fnv::{FnvHashMap, FnvHashSet};
use hlt::constants::*;
use hlt::state::*;

pub struct Scout {
    ships: FnvHashMap<ID, Vec<Ship>>,
    groups: Vec<Vec<Ship>>,
    distress: FnvHashSet<ID>,
    assist: FnvHashMap<ID, usize>,
    assign: FnvHashMap<ID, ID>,
    planets: FnvHashMap<ID, Vec<Planet>>,
}

impl Scout {
    pub fn new(id: ID, ships: &Ships, planets: &Planets) -> Self {
        let mut ordered_ships = FnvHashMap::default();
        let mut ordered_planets = FnvHashMap::default();
        let mut assign = FnvHashMap::default();
        let mut assist = FnvHashMap::default();
        let mut groups = Vec::new();
        let mut nearby = Vec::new();
        for ship in ships.values() {

            // Planets in order of distance
            let mut p = planets.values()
                .cloned()
                .collect::<Vec<_>>();
            p.sort_unstable_by(|a, b| {
                ship.distance_to(&a).partial_cmp(
                &ship.distance_to(&b)).unwrap()
            });
            ordered_planets.insert(ship.id, p);

            // Ships in order of distance
            let mut s = ships.values()
                .cloned()
                .collect::<Vec<_>>();
            s.sort_unstable_by(|a, b| {
                ship.distance_to(&a).partial_cmp(
                &ship.distance_to(&b)).unwrap()
            });

            // Nearby allies (including self)
            let group = s.iter()
                .filter(|ally| ally.owner == ship.owner)
                .filter(|ally| ship.distance_to(ally) < 7.0)
                .cloned()
                .collect::<Vec<_>>();

            s.remove(0);
            nearby.push(group);
            ordered_ships.insert(ship.id, s);
        }

        // Assign groups to greedily maximize ships per group
        nearby.sort_unstable_by(|a, b| { b.len().cmp(&a.len()) });

        for group in nearby {
            let unassigned = group.into_iter()
                .filter(|ship| !assign.contains_key(&ship.id))
                .collect::<Vec<_>>();

            if unassigned.len() == 0 { continue }

            let group = groups.len();
            for ship in &unassigned { assign.insert(ship.id, group); }
            groups.push(unassigned);
        }

        // Find groups in distress
        let mut distress = FnvHashSet::default();
        for ship in ships.values().filter(|ship| ship.owner == id) {
            let nearby = ordered_ships[&ship.id].iter()
                .filter(|other| !other.is_docked())
                .filter(|other| other.owner != id)
                .nth(0);

            if let Some(enemy) = nearby {
                let allies = groups[assign[&ship.id]].iter()
                    .filter(|ally| !ally.is_docked())
                    .count();
                let enemies = groups[assign[&enemy.id]].iter()
                    .filter(|enemy| !enemy.is_docked())
                    .count();
                let radius = if ship.is_docked() { DEFEND_RADIUS } else { COMBAT_RADIUS };
                if ship.distance_to(&enemy) < radius && enemies >= allies {
                    distress.insert(assign[&ship.id]);
                    assist.insert(assign[&ship.id], enemies - allies);
                }
            }
        }

        Scout { ships: ordered_ships, planets: ordered_planets,
                distress, groups, assign, assist }
    }

    pub fn is_distressed(&self, group: ID) -> bool {
        self.distress.contains(&group)
    }

    pub fn assist(&mut self, ship: &Ship, n: usize) {
        let group = self.assign[&ship.id];
        let m = self.assist[&group];
        self.assist.insert(group, m - n);
    }

    pub fn nearest_ally(&self, ship: &Ship) -> Option<&Ship> {
        self.ships[&ship.id].iter()
            .filter(|other| other.owner == ship.owner)
            .filter(|other| self.assign[&ship.id] != self.assign[&other.id])
            .filter(|other| !self.distress.contains(&self.assign[&other.id]))
            .nth(0)
    }

    pub fn nearest_enemy(&self, ship: &Ship) -> &Ship {
        self.ships[&ship.id].iter()
            .filter(|other| other.owner != ship.owner)
            .nth(0).expect("No enemies remaining")
    }

    pub fn nearest_dock(&self, ship: &Ship) -> Option<&Planet> {
        self.planets[&ship.id].iter()
            .filter(|planet| {
                ship.distance_to(planet) < planet.rad + DOCK_RADIUS - EPSILON
            })
            .filter(|planet| planet.has_spots())
            .filter(|planet| !planet.is_enemy(ship.owner))
            .min_by(|a, b| {
                ship.distance_to(a).partial_cmp(
                &ship.distance_to(b)).unwrap()
            }).and_then(|planet| {
                if ship.distance_to(&self.nearest_enemy(ship)) < 35.0 {
                    None
                } else { Some(planet) }
            })
    }

    pub fn nearest_distress(&self, ship: &Ship, d: f64) -> Option<(Ship, usize)> {
        self.ships[&ship.id].iter()
            .take_while(|other| ship.distance_to(other) < d)
            .filter(|other| other.id == ship.id)
            .filter(|other| other.owner == ship.owner)
            .filter(|other| self.distress.contains(&self.assign[&other.id]))
            .filter(|other| self.assist[&self.assign[&other.id]] > 0)
            .map(|other| (other.clone(), self.assist[&self.assign[&other.id]]))
            .nth(0)
    }

    pub fn nearest_target(&self, ship: &Ship, d: f64) -> Option<&Ship> {
        self.ships[&ship.id].iter()
            .take_while(|other| ship.distance_to(other) < d)
            .filter(|other| other.owner != ship.owner)
            .filter(|other| other.is_docked())
            .nth(0)
    }

    pub fn nearest_planet(&self, ship: &Ship, d: f64) -> Option<&Planet> {
        self.planets[&ship.id].iter()
            .take_while(|planet| ship.distance_to(planet) < d)
            .filter(|planet| planet.has_spots())
            .filter(|planet| !planet.is_enemy(ship.owner))
            .nth(0)
    }

    pub fn groups(&self, id: ID) -> Vec<(usize, Vec<Ship>)> {
        self.groups.iter()
            .enumerate()
            .filter(|&(_, ref ships)| ships[0].owner == id)
            .map(|(group, ships)| (group, ships.clone()))
            .collect::<Vec<_>>()
    }
}
