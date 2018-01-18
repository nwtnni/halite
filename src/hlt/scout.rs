use fnv::{FnvHashMap, FnvHashSet};
use std::collections::BinaryHeap;
use hlt::constants::*;
use hlt::state::*;
use hlt::tactic::*;

pub struct Scout {
    ships: FnvHashMap<ID, Vec<Ship>>,
    planets: FnvHashMap<ID, Vec<Planet>>,
    assign: FnvHashMap<ID, ID>,
    groups: Vec<Vec<Ship>>,
    distress: FnvHashSet<ID>,
    objectives: FnvHashMap<ID, BinaryHeap<Tactic>>,
}

impl Scout {
    pub fn new(id: ID, ships: &Ships, planets: &Planets) -> Self {
        let mut ordered_ships = FnvHashMap::default();
        let mut ordered_planets = FnvHashMap::default();
        let mut assign = FnvHashMap::default();
        let mut objectives = FnvHashMap::default();
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
                }
            }
        }

        // Assign all objectives
        for (ship, ships) in ordered_ships {
            let mut resolved = FnvHashSet::default();
            let mut priority = BinaryHeap::new();
            let ship = ships[ship];
            if ship.owner != id { continue }
            for other in ships {
                if resolved.contains(&assign[&other.id]) { continue }
                else if ship.owner == other.owner {
                    priority.push(Tactic::Aid(ship.clone(), other.clone()));
                } else {
                    priority.push(Tactic::Attack(ship.clone(), other.clone()));
                }
                resolved.insert(assign[&other.id]);
            }
            objectives.insert(ship.id, priority);
        }

        for (ship, planets) in ordered_planets {
            let ship = ships[&ship];
            for planet in planets {
                objectives[&ship.id].push(Tactic::Dock(ship.clone(), planet.clone()));
            }
        }

        Scout { ships: ordered_ships, planets: ordered_planets,
                distress, groups, assign, objectives }
    }

    pub fn is_distressed(&self, group: ID) -> bool {
        self.distress.contains(&group)
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

    pub fn groups(&self, id: ID) -> Vec<(usize, Vec<Ship>)> {
        self.groups.iter()
            .enumerate()
            .filter(|&(_, ref ships)| ships[0].owner == id)
            .map(|(group, ships)| (group, ships.clone()))
            .collect::<Vec<_>>()
    }
}
