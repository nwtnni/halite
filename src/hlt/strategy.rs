use fnv::{FnvHashSet, FnvHashMap};
use hlt::command::*;
use hlt::state::*;

// Priorities:
// 1) Retreat when in distress
// 2) Reinforce nearby distress signal
// 3) Go for nearby docked enemy
// 4) Go for nearby docking site
// 5) Go for closest distress signal
pub fn step(s: &mut State, turn: i32) {
    let mut resolved = FnvHashSet::default();
    for (group, ships) in s.scout.groups(s.id) {
    }

    s.queue.flush();
}
