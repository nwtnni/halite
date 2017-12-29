extern crate halite;
use halite::game::*;
use halite::strategy::*;
use halite::navigate::*;

fn main() {
    let mut game = Game::new();
    let mut queue = CommandQueue::new();
    let mut strategy = Strategies::new();
    Game::send_ready();

    loop {
        game.update();
        for ship_id in &game.players[game.id].ships {
            let ship = &game.ships[ship_id];
            if is_docked(ship) { continue; }

            match strategy.get(ship.id) {
                None => {
                    let closest = game.planets.values()
                        .filter(|planet| planet.spots > planet.ships.len() as i32)
                        .min_by_key(|planet| {
                            let x = planet.x - ship.x;
                            let y = planet.y - ship.y;
                            (x*x + y*y) as i32
                        }).expect("No planets found");

                    if can_dock(ship, closest) {
                        queue.push(&dock(ship, closest));
                    } else {
                        strategy.set(ship.id, Strategy::Dock(closest.id));
                        queue.push(&navigate(&mut game.grid, ship, closest));
                    }
                }
                Some(Strategy::Dock(id)) => {
                    let planet = &game.planets[&id];
                    if can_dock(ship, planet) {
                        queue.push(&dock(ship, planet));
                    } else {
                        queue.push(&navigate(&mut game.grid, ship, planet));
                    }
                }
            }
        }
        queue.flush();
    }
}
