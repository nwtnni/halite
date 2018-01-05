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
            .sum::<i32>()*3
    });
    ships.sort_unstable_by(|a, b| {
        a.distance_to(&sorted[0]).partial_cmp(
        &b.distance_to(&sorted[0])).unwrap()
    });

    // Let each ship make an independent decision
    for ship in &ships {

        if ship.is_docked() {
            s.plan.set(ship.id, Tactic::Dock(s.docked[&ship.id]));
            continue
        }

        // Make sure planet isn't occupied
        let closest = if s.plan.docking_at(sorted[0].id) >= sorted[0].spots {
            sorted[1]
        } else { sorted[0] };

        // If we've been given orders, follow along
        if let Some(_) = s.plan.has_target(ship.id) {
            continue
        }

        // Inside docking range: check if threats nearby
        let mut near = enemies.iter()
            .filter(|&enemy| ship.distance_to(enemy) < 70.0)
            .collect::<Vec<_>>();
        near.sort_unstable_by_key(|&enemy| ship.distance_to(enemy) as i32);

        // If we're outside of docking range
        if !ship.in_docking_range(closest) && near.len() == 0 {
            s.plan.set(ship.id, Tactic::Travel(closest.id));
            continue
        }

        // If no threats nearby, just dock
        if near.len() == 0 {
            if ship.in_docking_range(closest) {
                s.docked.insert(ship.id, closest.id);
                s.plan.set(ship.id, Tactic::Dock(closest.id));
            } else {
                s.plan.set(ship.id, Tactic::Travel(closest.id));
            }
            continue
        }

        // Check if docked enemy ships nearby
        let docked = near.iter()
            .filter(|&enemy| enemy.is_docked())
            .collect::<Vec<_>>();

        // If all enemy ships docked
        if docked.len() == near.len() {
            let enemy = &docked[0];
            for ship in &ships {
                s.plan.set(ship.id, Tactic::Attack(enemy.id));
            }
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

    // Assign ships to nearby planets
    for ship in &ships {
        let closest = &s.planets.values()
            .filter(|planet| {
                if !planet.is_enemy(s.id) {
                    planet.spots() > s.plan.docking_at(planet.id)
                } else { true }
            })
            .filter_map(|planet| {
                let o = s.plan.traveling_to(planet.id);
                let e = s.grid.near_enemies(&planet, planet.rad + 14.0, &s.ships)
                    .len() as i32;
                if o <= e { Some((planet, e)) }
                else { None }
            })
            .min_by_key(|&(planet, e)| {
                let a = s.grid.near_allies(&planet, planet.rad + 14.0, &s.ships)
                    .len() as i32;
                (e - a) + (ship.distance_to(&planet) as i32)
            });
        if let &Some((planet, _)) = closest {
            let e = s.grid.near_enemies(&planet, planet.rad + 35.0, &s.ships);
            let a = s.grid.near_allies(&planet, planet.rad + 14.0, &s.ships)
                .into_iter()
                .filter(|ally| !ally.is_docked()).count();
            if !ship.in_docking_range(planet) {
                s.plan.set(ship.id, Tactic::Travel(planet.id));
            } else if e.len() < a {
                s.plan.set(ship.id, Tactic::Dock(planet.id));
            } else {
                s.plan.set(ship.id, Tactic::Attack(e[0].id));
            }
        }
    }

    // Assign ships to attack in smaller skirmishes
    for ship in &ships {
        let allies = s.grid.near_allies(&ship, 14.0, &s.ships);
        let enemies = s.grid.near_enemies(&ship, 21.0, &s.ships);
        let docking = enemies.iter()
            .filter(|enemy| enemy.is_docked())
            .collect::<Vec<_>>();

        if enemies.len() > 0 && allies.len() >= enemies.len() - docking.len() {
            let mut n = 0;
            while let Some(target) = docking.get(n) {
                if s.plan.attacking(target.id) < 4 {
                    s.plan.set(ship.id, Tactic::Attack(target.id));
                    break
                }
                n += 1;
            }
        } else if enemies.len() - docking.len() > 0 {
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
                    .len() as i32 > s.plan.defending(planet.id)
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
                s.plan.traveling_to(planet.id) < e as i32
            })
            .min_by(|a, b| {
                ship.distance_to(a).partial_cmp(&ship.distance_to(b)).unwrap()
            });
        if let Some(planet) = closest {
            s.plan.set(ship.id, Tactic::Travel(planet.id));
        }
    }

    // Enemy planets that aren't being harassed
    let victims = &s.planets.values()
        .filter(|planet| planet.is_enemy(s.id) && planet.docked() < 3)
        .filter(|planet| !s.plan.is_victim(planet.id))
        .collect::<Vec<_>>();

    // Assign our closest ships to harass
    for victim in victims {
        let nearby = s.grid.near_allies(victim, 70.0, &s.ships)
            .iter()
            .min_by(|a, b| {
                a.distance_to(victim).partial_cmp(
                &b.distance_to(victim)).unwrap()
            }).cloned();
        if let Some(ally) = nearby {
            s.plan.set(ally.id, Tactic::Harass(victim.id));
        }
    }}
    Plan::execute(s);
}

/* Late Game Goals
 * - Clump up to take out enemy planets
 * - Defend from enemy attacks
 * - Hide when only a few ships are left(?)
 */
