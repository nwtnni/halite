use hlt::command::*;
use hlt::state::*;

pub fn step(s: &mut State, turn: i32) {
    let ships = s.ships.values()
        .filter(|ship| ship.owner == s.id)
        .filter(|ship| !s.docked.contains_key(&ship.id))
        .collect::<Vec<_>>();

    for ship in ships {
    }
}
