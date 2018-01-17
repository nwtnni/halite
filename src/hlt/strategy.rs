use hlt::command::*;
use hlt::state::*;

pub fn step(s: &mut State, turn: i32) {
    let ships = s.ships.values()
        .filter(|ship| ship.owner == s.id)
        .filter(|ship| !s.docked.contains_key(&ship.id))
        .collect::<Vec<_>>();

    for ship in ships {
        let planets = s.scout.get_planets(ship.id)
            .into_iter()
            .filter(|planet| ship.distance_to(planet) < 70.0)
            .collect::<Vec<_>>();

        for planet in planets {

        }
    }
}
