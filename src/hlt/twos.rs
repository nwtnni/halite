use hlt::constants::*;
use hlt::state::*;
use hlt::strategy::*;
use hlt::value::*;

pub fn step(s: &mut State, turn: i32) {
    middle(s);
    s.queue.flush();
}

/* Mid Game Goals
 * - Harass enemy planets
 * - Defend from enemy attacks
 * - Expand own territory
 */
fn middle(s: &mut State) {{
    let player = &s.players[s.id];
    let (docked, ships): (Vec<_>, Vec<_>) = player.ships.iter()
        .map(|ship| &s.ships[&ship])
        .cloned()
        .partition(|ship| ship.is_docked());

    for ship in docked {
        s.plan.set(ship.id, Tactic::Dock(s.docked[&ship.id]));
    }

    info!("Prioritizing!");
    for &(ship, planet) in &prioritize(s) {
        info!("Trying to send {} and buddies to {}", ship, planet);
        let ship = &s.ships[&ship];
        if !s.plan.is_available(ship.id) { continue }

        let planet = &s.planets[&planet];
        let mut allies = s.grid.near_allies(&ship, ASSEMBLE_RADIUS, &s.ships)
            .into_iter()
            .filter(|ally| s.plan.is_available(ally.id))
            .collect::<Vec<_>>();
        allies.insert(0, ship);

        let a = allies.len() as i32;
        let e = s.grid.near_enemies(&planet, planet.rad + SCAN_RADIUS, &s.ships)
            .into_iter()
            .filter(|enemy| !enemy.is_docked())
            .count() as i32;

        let (min, max, tactic) = if planet.is_free() {

            (1, planet.spots - s.plan.docking_at(planet.id) + e,
             Tactic::Travel(planet.id))

        } else if planet.is_owned(s.id) && planet.has_spots() && a >= e {

            (1, e + planet.spots - s.plan.docking_at(planet.id),
            Tactic::Travel(planet.id))

        } else if planet.is_owned(s.id) && a >= e {

            (a, e * 2 - s.plan.defending(planet.id),
            Tactic::Defend(planet.id))

        } else if planet.is_owned(s.id) && a < e {

            (2, e * 2 - s.plan.defending(planet.id),
            Tactic::Defend(planet.id))

        } else {

            (4, e * 2 - s.plan.attacking(planet.id),
            Tactic::Attack(planet.id))

        };

        if a < min { continue }
        let n = if a < max { a } else { max };
        info!("Succesfully sent {} ships to {}", n, planet.id);
        for i in 0..n {
            s.plan.set(allies[i as usize].id, tactic);
        }
    }}
    info!("1");
    Plan::execute(s);
    info!("2");
}

/* Late Game Goals
 * - Clump up to take out enemy planets
 * - Defend from enemy attacks
 * - Hide when only a few ships are left(?)
 */
