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

        let ships = ships.into_iter()
            .filter(|ship| !ship.is_docked())
            .collect::<Vec<_>>();
        if ships.len() == 0 { continue }

        /*
         *
         * Close Range Tactics
         *
         */

        // Retreat to ally, or away from enemy
        if s.scout.is_distressed(group) {
            info!("Distress signal on turn {} for group {:?}", turn, ships);
            for ship in ships {
                let ally = s.scout.nearest_ally(&ship);
                if let Some(ally) = ally {
                    s.queue.push(&navigate_to_ally(&mut s.grid, &ship, &ally));
                } else {
                    let enemy = s.scout.nearest_enemy(&ship);
                    s.queue.push(&navigate_from_enemy(&mut s.grid, &ship, &enemy));
                }
            }
            continue
        }

        // Check distance of distress signal
        if let Some((ally, required)) = s.scout.nearest_distress(&ships[0], 35.0) {
            info!("Responding to distress signal on turn {} for group {:?}", turn, ships);
            let mut n = 0;
            for ship in &ships {
                if n >= required { break }
                resolved.insert(ship.id);
                s.queue.push(&navigate_to_ally(&mut s.grid, &ship, &ally));
                n += 1;
            }
            s.scout.assist(&ally, n);
        }

        // Keep going without assisting ships
        let ships = ships.into_iter()
            .filter(|ship| !resolved.contains(&ship.id))
            .collect::<Vec<_>>();
        if ships.len() == 0 { continue }

        // Nearby docked enemy
        if let Some(enemy) = s.scout.nearest_target(&ships[0], 35.0) {
            info!("Attacking docked enemy {} on turn {} for group {:?}", enemy.id, turn, ships);
            for ship in ships {
                s.queue.push(&navigate_to_enemy(&mut s.grid, &ship, &enemy));
            }
            continue
        }

        // Dock if we can
        if let Some(planet) = s.scout.nearest_dock(&ships[0]) {
            info!("Docking on turn {} for group {:?}", turn, ships);
            let mut n = 0;
            for ship in &ships {
                if n < planet.spots() {
                    resolved.insert(ship.id);
                    s.queue.push(&dock(&ship, &planet));
                    n += 1;
                } else { break }
            }
        }

        // Keep going without docking ships
        let ships = ships.into_iter()
            .filter(|ship| !resolved.contains(&ship.id))
            .collect::<Vec<_>>();
        if ships.len() == 0 { continue }

        /*
         *
         * Long Range Tactics
         *
         */

        // Nearby enemy to fight
        if ships[0].distance_to(&s.scout.nearest_enemy(&ships[0])) < 35.0 {
            for ship in &ships {
                s.queue.push(
                    &navigate_to_enemy(&mut s.grid, &ship, &s.scout.nearest_enemy(&ships[0]))
                );
            }
            continue
        }

        // Farther docking sites
        if let Some(planet) = s.scout.nearest_planet(&ships[0], 70.0) {
            info!("Traveling to {} on turn {} for group {:?}", planet.id, turn, ships);
            for ship in ships {
                s.queue.push(&navigate_to_planet(&mut s.grid, &ship, &planet));
            }
            continue
        }

        // Any distress signal
        if let Some((ally, required)) = s.scout.nearest_distress(&ships[0], 500.0) {
            info!("Responding to distress signal on turn {} for group {:?}", turn, ships);
            let mut n = 0;
            for ship in &ships {
                if n >= required { break }
                resolved.insert(ship.id);
                s.queue.push(&navigate_to_ally(&mut s.grid, &ship, &ally));
                n += 1;
            }
            s.scout.assist(&ally, n);
        }

        // Keep going without assisting ships
        let ships = ships.into_iter()
            .filter(|ship| !resolved.contains(&ship.id))
            .collect::<Vec<_>>();
        if ships.len() == 0 { continue }

        // Any docked enemy
        if let Some(enemy) = s.scout.nearest_target(&ships[0], 500.0) {
            info!("Attacking docked enemy {} on turn {} for group {:?}", enemy.id, turn, ships);
            for ship in ships {
                s.queue.push(&navigate_to_enemy(&mut s.grid, &ship, &enemy));
            }
            continue
        }
    }

    s.queue.flush();
}
