use hlt::constants::*;
use hlt::state::*;
use hlt::tactic::*;

pub fn step(s: &mut State, turn: i32) {

    for ship in s.ships.values() {
        let closest = s.planets.values()
            .min_by(|a, b| {
                ship.distance_to(a).partial_cmp(
                &ship.distance_to(b)).unwrap()
            }).unwrap();

        if closest.is_enemy(s.id) {
            s.tactics.set(ship.id, Tactic::Raid(closest.id));
        } else if !closest.is_owned(s.id) || closest.has_spots() {
            s.tactics.set(ship.id, Tactic::Dock(closest.id));
        } else {
            s.tactics.set(ship.id, Tactic::Defend(closest.id));
        }
    }

    Tactics::execute(s);
}
