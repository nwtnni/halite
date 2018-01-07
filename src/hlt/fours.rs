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
        if s.plan.is_attacking(ship.id) {
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
fn middle(s: &mut State) {
}
