use hlt::command::*;
use hlt::state::*;
use hlt::strategy::*;

pub fn step(s: &mut State, turns: i32) {
    let allies = s.ships.values()
        .filter(|ship| ship.owner == s.id) 
        .count();
    if allies <= 3 {
        early(s)
    } else if s.ships.len() < 50 {
        middle(s)
    } else {
        late(s)
    }
    s.queue.flush();
}

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
        (((planet.x - s.width/2.0) * 
        (planet.y - s.height/2.0) /
        planet.spots as f64).abs() +
        (planet.y - ya).hypot(planet.x - xa)) as i32
    });
    ships.sort_unstable_by(|a, b| {
        a.distance_to(&sorted[0]).partial_cmp(
        &b.distance_to(&sorted[0])).unwrap()
    });

    // Let each ship make an independent decision
    for ship in &ships {

        if ship.is_docked() { continue }

        // Make sure planet isn't occupied
        let closest = if s.plan.docking_at(sorted[0].id) >= sorted[0].spots {
            sorted[1] 
        } else { sorted[0] };

        // If we've been given orders, follow along
        if let Some(_) = s.plan.has_target(ship.id) {
            continue
        }

        // If we're outside of docking range
        if !ship.in_docking_range(closest) && s.plan.strategy == Strategy::Neutral {
            s.plan.set(ship.id, Tactic::Travel(closest.id));
            continue
        }

        // Inside docking range: check if threats nearby
        let mut near = enemies.iter()
            .filter(|&enemy| ship.distance_to(enemy) < 49.0)
            .collect::<Vec<_>>();
        near.sort_unstable_by_key(|&enemy| ship.distance_to(enemy) as i32);

        // If no threats nearby, just dock
        if near.len() == 0 {
            s.plan.strategy = Strategy::Neutral;
            if ship.in_docking_range(closest) {
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
            s.plan.strategy = Strategy::Aggressive;
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

fn middle(s: &mut State) {
    unimplemented!();
}

fn late(s: &mut State) {
    unimplemented!();
}
