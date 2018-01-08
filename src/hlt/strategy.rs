use hlt::constants::*;
use hlt::state::*;
use hlt::tactics::*;

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
    let (mut ships, enemies): (Vec<_>, Vec<_>) = s.ships.values()
        .partition(|ship| ship.owner == s.id);

    // Find midpoint of group
    let a = ships.len() as f64;
    let (xa, ya) = ships.iter().map(|ship| (ship.x, ship.y))
        .fold((0.0, 0.0), |(x, y), (xs, ys)| (x + xs, y + ys));
    let (xa, ya) = (xa / a, ya / a);

    // Prioritize closest planet to us and center
    let mut sorted = s.planets.values().collect::<Vec<_>>();
    sorted.sort_unstable_by_key(|planet| {
        (planet.y - ya).hypot(planet.x - xa) as i32 -
        s.grid.near_planets(planet, 35.0, &s.planets)
            .into_iter()
            .map(|planet| planet.spots)
            .sum::<i32>()*4
    });
    ships.sort_unstable_by(|a, b| {
        a.distance_to(&sorted[0]).partial_cmp(
        &b.distance_to(&sorted[0])).unwrap()
    });

    // Let each ship make an independent decision
    for ship in &ships {

        if ship.is_docked() {
            s.tactics.set(ship.id, Tactic::Dock(s.docked[&ship.id]));
            continue
        }

        // Make sure planet isn't occupied
        let mut n = 0;
        while s.tactics.docking_at(sorted[n].id) >= sorted[n].spots {
            n += 1;
        }
        let closest = sorted[n];

        // If we've been given orders, follow along
        if let Some(_) = s.tactics.has_target(ship.id) {
            continue
        }

        // Check if threats nearby
        let mut near = enemies.iter()
            .filter(|&enemy| ship.distance_to(enemy) < 56.0)
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
                s.tactics.set(ship.id, Tactic::Dock(closest.id));
            } else {
                s.tactics.set(ship.id, Tactic::Travel(closest.id));
            }
            continue
        }

        // If all enemy ships docked, attack
        if docked.len() == near.len() && s.docked.len() == 0 {
            let enemy = &docked[0];
            for ship in &ships {
                if !s.docked.contains_key(&ship.id) {
                    s.tactics.set(ship.id, Tactic::Attack(enemy.id));
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
                if !s.docked.contains_key(&ship.id) {
                    s.tactics.set(ship.id, Tactic::Attack(enemy.id));
                }
            }
        }
    }}
    Tactics::execute(s);
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
            s.tactics.set(ship.id, Tactic::Dock(s.docked[&ship.id]));
        }
    }

    // Assign ships to nearby planets
    for ship in &ships {
        let closest = &s.planets.values()
            .filter(|planet| {
                if !planet.is_enemy(s.id) {
                    planet.spots() > s.tactics.docking_at(planet.id)
                } else { true }
            })
            .filter(|&planet| {
                let o = s.tactics.traveling_to(planet.id);
                let e = s.grid.near_enemies(&planet, planet.rad + 14.0, &s.ships)
                    .len() as i32;
                o / 2 <= e
            })
            .min_by_key(|&planet| {
                ship.distance_to(&planet) as i32
            });
        if let &Some(planet) = closest {
            let e = s.grid.near_enemies(&planet, planet.rad + 35.0, &s.ships)
                .into_iter()
                .filter(|enemy| !enemy.is_docked()).collect::<Vec<_>>();
            let a = s.grid.near_allies(&planet, planet.rad + 14.0, &s.ships)
                .into_iter()
                .filter(|ally| !ally.is_docked()).count();
            if !ship.in_docking_range(planet) {
                s.tactics.set(ship.id, Tactic::Travel(planet.id));
            } else if e.len() < a && !planet.is_enemy(s.id) && planet.has_spots() {
                s.docked.insert(ship.id, planet.id);
                s.tactics.set(ship.id, Tactic::Dock(planet.id));
            } else if e.len() > 0 {
                s.tactics.set(ship.id, Tactic::Attack(e[0].id));
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
                if s.tactics.attacking(target.id) < 8 {
                    s.tactics.set(ship.id, Tactic::Attack(target.id));
                    break
                }
                n += 1;
            }
        } else if allies.len() >= enemies.len() {
            let mut n = 0;
            while let Some(&target) = enemies.get(n) {
                if s.tactics.attacking(target.id) < 8 {
                    s.tactics.set(ship.id, Tactic::Attack(target.id));
                    break
                }
                n += 1;
            }
        } else if enemies.len() > 0 {
            let enemy = enemies[0].id;
            s.tactics.set(ship.id, Tactic::Retreat(enemy));
        }
    }

    // Assign ships to defend
    for ship in &ships {
        let distress = s.planets.values()
            .filter(|planet| planet.is_owned(s.id))
            .filter(|planet| {
                s.grid.near_enemies(planet, defense_radius(planet), &s.ships)
                    .len() as i32 > s.tactics.defending(planet.id) / 2
            }).min_by(|a, b| {
                ship.distance_to(a).partial_cmp(&ship.distance_to(b)).unwrap()
            });

        if let Some(distress) = distress {
            s.tactics.set(ship.id, Tactic::Defend(distress.id));
        }
    }

    // When in doubt, go attack the enemy
    for ship in &ships {
        if !s.tactics.is_available(ship.id) { continue }
        let closest = s.planets.values()
            .filter(|planet| planet.is_enemy(s.id))
            .filter(|planet| {
                let e = s.grid.near_enemies(planet, 100.0, &s.ships).len();
                s.tactics.traveling_to(planet.id) / 2 < e as i32
            })
            .min_by(|a, b| {
                ship.distance_to(a).partial_cmp(&ship.distance_to(b)).unwrap()
            });
        if let Some(planet) = closest {
            s.tactics.set(ship.id, Tactic::Travel(planet.id));
        }
    }

    // Enemy planets that aren't being harassed
    let victims = &s.planets.values()
        .filter(|planet| planet.is_enemy(s.id) && planet.docked() < 3)
        .filter(|planet| !s.tactics.is_victim(planet.id))
        .collect::<Vec<_>>();

    // Assign our closest ships to harass
    for victim in victims {
        let nearby = s.grid.near_allies(victim, 100.0, &s.ships)
            .iter()
            .min_by(|a, b| {
                a.distance_to(victim).partial_cmp(
                &b.distance_to(victim)).unwrap()
            }).cloned();
        if let Some(ally) = nearby {
            s.tactics.set(ally.id, Tactic::Harass(victim.id));
        }
    }}
    Tactics::execute(s);
}

/* Late Game Goals
 * - Clump up to take out enemy planets
 * - Defend from enemy attacks
 * - Hide when only a few ships are left(?)
 */
