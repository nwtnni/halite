use fnv::FnvHashSet;
use hlt::command::*;
use hlt::state::*;

// Priorities:
// 1) Retreat when in distress
// 2) Reinforce nearby distress signal
// 3) Go for nearby docked enemy
// 4) Go for nearby docking site
// 5) Go for closest distress signal
pub fn step(s: &mut State, turn: i32) {
    let mut docking = FnvHashSet::default();
    for (group, ships) in s.scout.groups(s.id) {

        let ships = ships.into_iter()
            .filter(|ship| !ship.is_docked())
            .collect::<Vec<_>>();
        if ships.len() == 0 { continue }

        // Retreat to ally, or away from enemy
        if s.scout.is_distressed(group) {
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
        if let Some(ally) = s.scout.nearest_distress(&ships[0], 35.0) {
            for ship in ships {
                s.queue.push(&navigate_to_ally(&mut s.grid, &ship, &ally));
            }
            continue
        }

        // Dock if we can
        if let Some(planet) = s.scout.nearest_dock(&ships[0]) {
            let mut n = 0;
            for ship in &ships {
                if n + planet.spots() > planet.spots {
                    docking.insert(ship.id);
                    s.queue.push(&dock(&ship, &planet));
                    n += 1;
                } else { break }
            }
        }

        // Keep going without docking ships
        let ships = ships.into_iter()
            .filter(|ship| !docking.contains(&ship.id))
            .collect::<Vec<_>>();
        if ships.len() == 0 { continue }

        // Nearby docked enemy
        if let Some(enemy) = s.scout.nearest_target(&ships[0], 35.0) {
            for ship in ships {
                s.queue.push(&navigate_to_enemy(&mut s.grid, &ship, &enemy));
            }
            continue
        }

        // Nearby docking site
        if let Some(planet) = s.scout.nearest_planet(&ships[0], 70.0) {
            for ship in ships {
                s.queue.push(&navigate_to_planet(&mut s.grid, &ship, &planet)); 
            } 
            continue
        }

        // Otherwise reinforce distress signal
        if let Some(ally) = s.scout.nearest_distress(&ships[0], 500.0) {
            for ship in ships {
                s.queue.push(&navigate_to_ally(&mut s.grid, &ship, &ally));
            }
            continue
        }
    }

    s.queue.flush();
}
