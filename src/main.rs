extern crate halite;
extern crate fnv;
use halite::state::*;
use halite::collision::Grid;
use halite::commander::*;
use halite::strategy::*;

fn main() {
    let mut game = Game::new();
    let mut queue = Queue::new();
    let mut strategy = Strategies::new();
    Game::send_ready("TestBot");

    loop {
        game.update();
        for ship_id in &game.players[game.id].ships {
            let ship = &game.ships[ship_id];
            if is_docked(ship) { continue; }

            match strategy.get(ship.id) {
                None => {
                    do_none(ship, &game.planets, &mut strategy,
                            &mut game.grid, &mut queue)
                },
                Some(Strategy::Dock(id)) => {
                    do_dock(ship, id, &game.planets,
                            &mut game.grid, &mut queue)
                },
                Some(Strategy::Attack(_id)) => {},
                Some(Strategy::Follow(_id)) => {},
            }
        }
        queue.flush();
    }
}

fn do_none(ship: &Ship,
           planets: &Planets,
           strategy: &mut Strategies,
           grid: &mut Grid,
           queue: &mut Queue) 
{
    let closest = closest_planet(ship, &planets, &strategy).unwrap();
    if can_dock(ship, closest) {
        queue.push(&dock(ship, closest));
    } else {
        strategy.set(ship.id, Strategy::Dock(closest.id));
        queue.push(&navigate(grid, ship, closest));
    }
}

fn do_dock(ship: &Ship,
           id: ID,
           planets: &Planets,
           grid: &mut Grid,
           queue: &mut Queue) 
{
    let planet = &planets[&id];
    if can_dock(ship, planet) {
        queue.push(&dock(ship, planet));
    } else {
        queue.push(&navigate(grid, ship, planet));
    }
}
