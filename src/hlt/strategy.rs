use hlt::constants::*;
use hlt::state::*;
use hlt::tactic::*;

pub fn step(s: &mut State, turn: i32) {{
    let allies = s.players[s.id].ships.len();
    if allies <= 3 && turn < 30 {
        early(s);
        return
    }
    middle(s);
}

fn early(s: &mut State) {{
    let (mut ships, enemies): (Vec<_>, Vec<_>) = s.ships.values()
        .partition(|ship| ship.owner == s.id);

    for (&ship, &planet) in s.docked.iter() {
        if !s.ships.contains_key(&ship) { continue }
        s.tactics.set(ship, Tactic::Dock(planet));
    }

    // Find midpoint of group
    let a = ships.len() as f64;
    let (xa, ya) = ships.iter().map(|ship| (ship.x, ship.y))
        .fold((0.0, 0.0), |(x, y), (xs, ys)| (x + xs, y + ys));
    let (xa, ya) = (xa / a, ya / a);

    // Prioritize closest planet to us and center
    let mut sorted = s.planets.values().collect::<Vec<_>>();

    if s.players.len() == 2 {
        sorted.sort_unstable_by_key(|planet| {
            (planet.y - ya).hypot(planet.x - xa) as i32 -
            s.planets.values()
                .filter(|other| {
                    (planet.x - other.x).hypot(planet.y - other.y) < other.rad + 35.0
                })
                .map(|planet| planet.spots as i32)
                .sum::<i32>()*4
        });
        ships.sort_unstable_by(|a, b| {
            a.distance_to(&sorted[0]).partial_cmp(
            &b.distance_to(&sorted[0])).unwrap()
        });
    }


    // Let each ship make an independent decision
    for ship in &ships {

        if ship.is_docked() { continue }

        if s.players.len() == 4 {
            sorted.sort_unstable_by_key(|planet| {
            ((planet.y - ship.y).hypot(planet.x - ship.x).powf(1.5) -
            (planet.x - s.width/2.0).abs() -
            (planet.y - s.height/2.0).abs()) as i32
            });
        }

        // Make sure planet isn't occupied
        let mut n = 0;
        while s.tactics.docking(sorted[n].id) >= sorted[n].spots {
            n += 1;
        }
        let closest = sorted[n];

        // If we've been given orders, follow along
        if s.tactics.is_attacking(ship.id) { continue }

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
            s.tactics.set(ship.id, Tactic::Dock(closest.id));
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

fn middle(s: &mut State) {

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

        for planet in planets.iter() {
            let &(ref allies, ref enemies) = s.scout.get_env(planet.id);
            let a = allies.iter()
                .filter(|ally| !ally.is_docked())
                .count();
            let e = enemies.iter()
                .filter(|enemy| !enemy.is_docked())
                .count();
            if planet.is_enemy(s.id) {
                s.tactics.set(ship.id, Tactic::Raid(planet.id));
                break
            } else if !planet.is_owned(s.id) || planet.has_spots() {
                if s.tactics.docking(planet.id) >= planet.spots || e > a/2 {
                    continue
                }
                s.tactics.set(ship.id, Tactic::Dock(planet.id));
                break
            } else if e > 0 {
                if s.tactics.defending(planet.id) >= e*2 {
                    continue
                }
                s.tactics.set(ship.id, Tactic::Defend(planet.id));
                break
            }
        }
    }}
    Tactics::execute(s);
}
