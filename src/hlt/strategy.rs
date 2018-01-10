use hlt::constants::*;
use hlt::state::*;
use hlt::tactic::*;

pub fn step(s: &mut State, turn: i32) {{

    let ships = s.players[s.id].ships.iter()
        .map(|ship| &s.ships[&ship])
        .filter(|&ship| !s.docked.contains_key(&ship.id))
        .cloned()
        .collect::<Vec<_>>();

    for ship in ships {
        let mut planets = s.planets.values()
            .filter(|planet| ship.distance_to(planet) < 70.0)
            .collect::<Vec<_>>();

        planets.sort_unstable_by(|a, b| {
            ship.distance_to(a).partial_cmp(&ship.distance_to(b)).unwrap()
        });

        for planet in planets {
            let &(_, ref enemies) = s.scout.get_env(planet.id);
            if planet.is_enemy(s.id) {
                s.tactics.set(ship.id, Tactic::Raid(planet.id));
                break
            } else if !planet.is_owned(s.id) || planet.has_spots() {
                s.tactics.set(ship.id, Tactic::Dock(planet.id));
                break
            } else if enemies.len() > 0 {
                s.tactics.set(ship.id, Tactic::Defend(planet.id));
                break
            }
        }
    }}
    Tactics::execute(s);
}
