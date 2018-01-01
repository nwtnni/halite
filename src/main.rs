extern crate fnv;
mod hlt;
use hlt::state::*;
use hlt::collision::{Grid};
use hlt::command::*;
use hlt::strategy::*;

fn main() {
    let mut game = Game::new();
    let mut queue = Queue::new();
    let mut strategy = Strategies::new();
    Game::send_ready("TestBot");

    loop {
        game.update();
        strategy.clean(&mut game.ships);
        for ship_id in &game.players[game.id].ships {
            let ship = &game.ships[ship_id];
            if ship.is_docked() { continue; }

            match strategy.get(ship.id) {
                None => {
                    do_none(game.id, ship, &game.ships, &game.planets,
                            &mut strategy, &mut game.grid, &mut queue)
                },
                Some(Strategy::Dock(target)) => {
                    do_dock(game.id, ship, target, &game.ships, &game.planets,
                            &mut strategy, &mut game.grid, &mut queue)
                },
                Some(Strategy::Attack(target)) => {
                    do_attack(ship, target, &game.ships,
                              &mut strategy, &mut game.grid, &mut queue)
                },
            }
        }
        queue.flush();
    }
}

fn assign_attack(ship: &Ship,
                 ships: &Ships,
                 strategy: &mut Strategies,
                 grid: &mut Grid,
                 queue: &mut Queue)
{
    if let Some(best) = best_target(ship, ships) {
        strategy.set(ship.id, Strategy::Attack(best.id));
        queue.push(&navigate(grid, ship, best));
    } else {
        return;
    }
}

fn do_none(id: ID,
           ship: &Ship,
           ships: &Ships,
           planets: &Planets,
           strategy: &mut Strategies,
           grid: &mut Grid,
           queue: &mut Queue)
{
    match best_planet(ship, &planets, &strategy) {
        None => {
            assign_attack(ship, ships, strategy, grid, queue);
        },
        Some(best) => {
            if best.has_spots() && !best.is_enemy(id) {
                if ship.in_docking_range(best) {
                    strategy.set(ship.id, Strategy::Dock(best.id));
                    queue.push(&dock(ship, best));
                } else {
                    strategy.set(ship.id, Strategy::Dock(best.id));
                    queue.push(&navigate(grid, ship, best));
                }
            } else {
                assign_attack(ship, ships, strategy, grid, queue);
            }
        }
    }
}

fn do_dock(id: ID,
           ship: &Ship,
           target: ID,
           ships: &Ships,
           planets: &Planets,
           strategy: &mut Strategies,
           grid: &mut Grid,
           queue: &mut Queue)
{
    if !planets.contains_key(&target) {
        return assign_attack(ship, ships, strategy, grid, queue);
    }
    let planet = &planets[&target];
    if planet.has_spots() && !planet.is_enemy(id) {
        if ship.in_docking_range(planet) {
            queue.push(&dock(ship, planet));
        }
        else {
            queue.push(&navigate(grid, ship, planet));
        }
    } else {
        assign_attack(ship, ships, strategy, grid, queue);
    }
}

fn do_attack(ship: &Ship,
             target: ID,
             ships: &Ships,
             strategy: &mut Strategies,
             grid: &mut Grid,
             queue: &mut Queue)
{
    if !ships.contains_key(&target) {
        return assign_attack(ship, ships, strategy, grid, queue);
    } else {
        queue.push(&navigate(grid, ship, &ships[&target]));
    }
}
