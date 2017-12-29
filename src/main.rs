extern crate halite;
use halite::game::*;

fn main() {
    let game = Game::new();
    Game::send_ready();

    loop {
        let map = &game.map;
        let player = &map.players[game.id];
        for ship_id in &player.ships {
            let _ship = &map.ships[ship_id];
        }
        unimplemented!();
    }
}
