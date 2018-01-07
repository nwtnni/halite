#![allow(unused_variables, dead_code)]
use hlt::state::*;
use hlt::constants::*;

pub fn prioritize(s: &State) -> Vec<(ID, ID)>{
    let mut best = Vec::new();
    let planets = s.planets.values().collect::<Vec<_>>();
    let ships = s.ships.values()
        .filter(|ship| {
            ship.owner == s.id && !ship.is_docked()
        }).collect::<Vec<_>>();

    for ship in &ships {
        let mut feasible = Vec::new();
        for planet in &planets {
            let e = s.grid.near_enemies(planet, planet.rad + SCAN_RADIUS, &s.ships)
                .into_iter()
                .filter(|enemy| !enemy.is_docked())
                .count() as f64;

            let mut a = s.grid.near_allies(ship, ASSEMBLE_RADIUS, &s.ships)
                .into_iter()
                .filter(|ally| !ally.is_docked())
                .collect::<Vec<_>>();
            a.push(ship);

            let (x, y) = a.iter()
                .map(|ship| (ship.x, ship.y))
                .fold((0.0, 0.0), |(x, y), (xs, ys)| (x + xs, y + ys));

            let a = a.len() as f64;
            let (x, y) = (x / a, y / a);
            let d = (planet.y - y).hypot(planet.x - x);

            if e > 0.0 && a < e * 2.0 { continue }
            let v = if planet.is_free() {
                -d - e
            } else if planet.is_enemy(s.id) {
                -d + ((planet.docked() - 2).pow(8) * 7) as f64
            } else if planet.has_spots() {
                -d + (planet.docked() * 7) as f64
            } else {
                -d + (planet.docked().pow(2) as f64 * e)
            };
            feasible.push((ship.id, planet.id, v))
        }
        feasible.sort_unstable_by(|&(_, _, v1), &(_, _, v2)| {
            v2.partial_cmp(&v1).expect("Invalid value")
        });
        feasible.truncate(5);
        best.extend(feasible);
    }
    best.sort_unstable_by(|&(_, _, v1), &(_, _, v2)| {
        v2.partial_cmp(&v1).expect("Invalid value")
    });
    best.into_iter()
        .map(|(s, p, _)| (s, p))
        .collect()
}
