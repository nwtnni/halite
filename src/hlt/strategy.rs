use hlt::constants::*;
use hlt::state::*;
use hlt::tactic::*;

pub fn step(s: &mut State, turn: i32) {{
    // Consistency of state (since docking ships aren't in game info)
    for (&ship, &planet) in s.docked.iter() {
        if !s.ships.contains_key(&ship) { continue }
        s.tactics.set(ship, Tactic::Dock(planet));
    }

    let ships = s.players[s.id].ships.iter()
        .map(|ship| &s.ships[&ship])
        .filter(|&ship| !s.docked.contains_key(&ship.id))
        .cloned()
        .collect::<Vec<_>>();

    for ship in ships {
        let mut planets = s.planets.values().collect::<Vec<_>>();
        planets.sort_unstable_by(|a, b| {
            ship.distance_to(a).partial_cmp(&ship.distance_to(b)).unwrap()
        });

        for planet in planets {
            let &(_, ref enemies) = s.scout.get_env(planet.id);
            if planet.is_enemy(s.id) {
                if s.tactics.raiding(planet.id) >= enemies.len()*2 {
                    continue
                }
                s.tactics.set(ship.id, Tactic::Raid(planet.id));
                break
            } else if !planet.is_owned(s.id) || planet.has_spots() {
                if s.tactics.docking(planet.id) >= enemies.len() + planet.spots {
                    continue
                }
                s.tactics.set(ship.id, Tactic::Dock(planet.id));
                break
            } else if enemies.len() > 0 {
                if s.tactics.defending(planet.id) >= enemies.len() {
                    continue
                }
                s.tactics.set(ship.id, Tactic::Defend(planet.id));
                break
            }
        }
    }}
    Tactics::execute(s);
}
