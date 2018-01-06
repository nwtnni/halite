use hlt::constants::*;
use hlt::state::*;
use hlt::strategy::*;

pub fn step(s: &mut State, turn: i32) {
    let allies = s.players[s.id].ships.len();
    if allies <= 3 && turn < 30 {
        early(s);
        s.queue.flush();
        return
    }
    middle(s);
    s.queue.flush();
}

/* Early Game Goals
 * - Establish dominance over the middle
 * - Defend from rushing enemies
 * - Rush docking enemies
 */
fn early(s: &mut State) {{
    let (ships, enemies): (Vec<_>, Vec<_>) = s.ships.values()
        .partition(|ship| ship.owner == s.id);

    // Let each ship make an independent decision
    for ship in &ships {

        // Prioritize closest planet to us
        let mut sorted = s.planets.values().collect::<Vec<_>>();
        sorted.sort_unstable_by_key(|planet| {
            ((planet.y - ship.y).hypot(planet.x - ship.x).powf(1.5) -
            (planet.x - s.width/2.0).abs() -
            (planet.y - s.height/2.0).abs()) as i32
        });

        if ship.is_docked() {
            s.plan.set(ship.id, Tactic::Dock(s.docked[&ship.id]));
            continue
        }

        // Make sure planet isn't occupied
        let mut n = 0;
        while s.plan.docking_at(sorted[n].id) >= sorted[n].spots {
            n += 1;
        }
        let closest = sorted[n];

        // If we've been given orders, follow along
        if let Some(_) = s.plan.has_target(ship.id) {
            continue
        }

        // Check if threats nearby
        let mut near = enemies.iter()
            .filter(|&enemy| ship.distance_to(enemy) < 35.0)
            .collect::<Vec<_>>();
        near.sort_unstable_by_key(|&enemy| ship.distance_to(enemy) as i32);

        // Check if docked enemy ships nearby
        let docked = near.iter()
            .filter(|&enemy| enemy.is_docked())
            .collect::<Vec<_>>();

        // If there are no enemies, proceed as usual
        if near.len() == 0 {
            if ship.in_docking_range(closest) {
                s.docked.insert(ship.id, closest.id);
                s.plan.set(ship.id, Tactic::Dock(closest.id));
            } else {
                s.plan.set(ship.id, Tactic::Travel(closest.id));
            }
            continue
        }

        // If all enemy ships docked, attack
        if docked.len() == near.len() && s.docked.len() == 0 {
            let enemy = &docked[0];
            for ship in &ships {
                if !s.docked.contains_key(&ship.id) {
                    s.plan.set(ship.id, Tactic::Attack(enemy.id));
                }
            }
            continue
        }

        // Otherwise fight off attacker
        else {
            let enemy = near.iter()
                .min_by_key(|&&enemy| ship.distance_to(enemy) as i32)
                .unwrap();
            for ship in &ships {
                s.plan.set(ship.id, Tactic::Attack(enemy.id));
            }
        }
    }}
    Plan::execute(s);
}

/* Mid Game Goals
 * - Harass enemy planets
 * - Defend from enemy attacks
 * - Expand own territory
 */
fn middle(s: &mut State) {{
    // Available ships
    let ships = s.players[s.id].ships.iter()
        .map(|ship| &s.ships[&ship])
        .filter(|&ship| !ship.is_docked())
        .cloned()
        .collect::<Vec<_>>();

    // Keep track of docked and docking ships
    for ship in &ships {
        if s.docked.contains_key(&ship.id) {
            s.plan.set(ship.id, Tactic::Dock(s.docked[&ship.id]));
        }
    }

    // Assign ships to nearby planets
    for ship in &ships {
        let closest = &s.planets.values()
            .filter(|planet| {
                if !planet.is_enemy(s.id) {
                    planet.spots() > s.plan.docking_at(planet.id)
                } else { true }
            })
            .filter(|&planet| {
                let o = s.plan.traveling_to(planet.id);
                let e = s.grid.near_enemies(&planet, planet.rad + 14.0, &s.ships)
                    .len() as i32;
                o / 2 <= e
            })
            .min_by_key(|&planet| {
                (ship.distance_to(&planet).powf(2.5) -
                (planet.x - s.width/2.0).powf(2.0) -
                (planet.y - s.height/2.0).powf(2.0)) as i32
            });
        if let &Some(planet) = closest {
            let (d, e): (Vec<_>, Vec<_>) = s.grid
                .near_enemies(&planet, planet.rad + 35.0, &s.ships)
                .into_iter()
                .partition(|enemy| enemy.is_docked());

            let a = s.grid.near_allies(&planet, planet.rad + 14.0, &s.ships)
                .into_iter()
                .filter(|ally| !ally.is_docked()).count();
            if !ship.in_docking_range(planet) {
                s.plan.set(ship.id, Tactic::Travel(planet.id));
            } else if e.len() < a && !planet.is_enemy(s.id) && planet.has_spots() {
                s.docked.insert(ship.id, planet.id);
                s.plan.set(ship.id, Tactic::Dock(planet.id));
            } else if d.len() > 0 {
                s.plan.set(ship.id, Tactic::Attack(d[0].id));
            } else if e.len() > 0 {
                s.plan.set(ship.id, Tactic::Attack(e[0].id));
            }
        }
    }

    // Assign ships to attack in smaller skirmishes
    for ship in &ships {
        let allies = s.grid.near_allies(&ship, 14.0, &s.ships);
        let enemies = s.grid.near_enemies(&ship, 14.0, &s.ships);
        let docking = enemies.iter()
            .filter(|enemy| enemy.is_docked())
            .collect::<Vec<_>>();

        if docking.len() > 0 {
            let mut n = 0;
            while let Some(target) = docking.get(n) {
                if s.plan.attacking(target.id) < 16 {
                    s.plan.set(ship.id, Tactic::Attack(target.id));
                    break
                }
                n += 1;
            }
        } else if allies.len() >= enemies.len() {
            let mut n = 0;
            while let Some(&target) = enemies.get(n) {
                if s.plan.attacking(target.id) < 8 {
                    s.plan.set(ship.id, Tactic::Attack(target.id));
                    break
                }
                n += 1;
            }
        } else if enemies.len() > 0 {
            let enemy = enemies[0].id;
            s.plan.set(ship.id, Tactic::Retreat(enemy));
        }
    }

    // Assign ships to defend
    for ship in &ships {
        let distress = s.planets.values()
            .filter(|planet| planet.is_owned(s.id))
            .filter(|planet| {
                s.grid.near_enemies(planet, defense_radius(planet), &s.ships)
                    .len() as i32 > s.plan.defending(planet.id) / 2
            }).min_by(|a, b| {
                ship.distance_to(a).partial_cmp(&ship.distance_to(b)).unwrap()
            });

        if let Some(distress) = distress {
            s.plan.set(ship.id, Tactic::Defend(distress.id));
        }
    }

    // When in doubt, go attack the enemy
    for ship in &ships {
        if !s.plan.is_available(ship.id) { continue }
        let closest = s.planets.values()
            .filter(|planet| planet.is_enemy(s.id))
            .filter(|planet| {
                let e = s.grid.near_enemies(planet, 100.0, &s.ships).len();
                s.plan.traveling_to(planet.id) / 2 < e as i32
            })
            .min_by(|a, b| {
                ship.distance_to(a).partial_cmp(&ship.distance_to(b)).unwrap()
            });
        if let Some(planet) = closest {
            s.plan.set(ship.id, Tactic::Travel(planet.id));
        }
    }}
    Plan::execute(s);
}

/* Late Game Goals
 * - Clump up to take out enemy planets
 * - Defend from enemy attacks
 * - Hide when only a few ships are left(?)
 */
