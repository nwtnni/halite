extern crate halite;
use halite::game::*;
use halite::navigate::Commander;

fn main() {
    let mut game = Game::new();
    let mut queue = CommandQueue::new();
    Game::send_ready();

    loop {
        for ship_id in &game.map.players[game.id].ships {
            let ship = &game.map.ships[ship_id];
            if Game::is_docked(ship) { continue; }
            let closest = game.map.planets.iter()
                .min_by_key(|planet| {
                    let x = planet.x - ship.x; 
                    let y = planet.y - ship.y;
                    (x*x + y*y) as i32
                }).expect("No planets found");

            if Game::can_dock(ship, closest) {
                queue.push(&Game::dock(ship, closest));
            } else {
                queue.push(&game.command(ship, closest));
            }
        }
        queue.flush();
        game.update();
    }
}
