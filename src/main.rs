extern crate halite;
extern crate fnv;
use halite::state::*;
use halite::collision::{Grid, within};
use halite::constants::{SHIP_RADIUS, ASSEMBLE_RADIUS};
use halite::commander::*;
use halite::strategy::*;

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
            if is_docked(ship) { continue; }

            match strategy.get(ship.id) {
                None => {
                    do_none(game.id, ship, &game.ships, &game.planets,
                            &mut strategy, &mut game.grid, &mut queue)
                },
                Some(Strategy::Dock(target)) => {
                    do_dock(ship, target, &game.planets,
                            &mut strategy, &mut game.grid, &mut queue)
                },
                Some(Strategy::Attack(target)) => {
                    do_attack(ship, target, &game.ships,
                              &mut strategy, &mut game.grid, &mut queue)
                },
                Some(Strategy::Follow(target)) => {
                    do_follow(ship, target, &game.ships,
                              &mut strategy, &mut game.grid, &mut queue)
                },
                Some(Strategy::Assemble(target)) => {
                    do_assemble(ship, target, &game.ships,
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
    if let Some(attacker) = strategy.attack_group(ship, ships) {
        strategy.set(ship.id, Strategy::Assemble(attacker));
        queue.push(&navigate(grid, ship, &ships[&attacker]));
    } else {
        if let Some(best) = best_target(ship, ships) {
            strategy.set(ship.id, Strategy::Attack(best.id));
            queue.push(&navigate(grid, ship, best));
        }
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
            if is_enemy(id, best) {
                assign_attack(ship, ships, strategy, grid, queue);
            } else {
                if can_dock(ship, best) {
                    queue.push(&dock(ship, best));
                } else {
                    strategy.set(ship.id, Strategy::Dock(best.id));
                    queue.push(&navigate(grid, ship, best));
                }
            }
        }
    }
}

fn do_dock(ship: &Ship,
           target: ID,
           planets: &Planets,
           strategy: &mut Strategies,
           grid: &mut Grid,
           queue: &mut Queue)
{
    if !planets.contains_key(&target) {
        return strategy.clear(ship.id);
    }
    let planet = &planets[&target];
    if can_dock(ship, planet) {
        queue.push(&dock(ship, planet));
    } else {
        queue.push(&navigate(grid, ship, planet));
    }
}

fn do_assemble(ship: &Ship,
               target: ID,
               ships: &Ships,
               strategy: &mut Strategies,
               grid: &mut Grid,
               queue: &mut Queue)
{
    if !ships.contains_key(&target) {
        return assign_attack(ship, ships, strategy, grid, queue);
    }
    let target = &ships[&target];
    if within((ship.x, ship.y), SHIP_RADIUS,
              (target.x, target.y), SHIP_RADIUS, ASSEMBLE_RADIUS) {
        strategy.set(ship.id, Strategy::Follow(target.id));
    } else {
        queue.push(&navigate(grid, ship, target));
    }
}

fn do_attack(ship: &Ship,
             target: ID,
             ships: &Ships,
             strategy: &mut Strategies,
             grid: &mut Grid,
             queue: &mut Queue)
{
    if strategy.assembling(ship.id, ships) > 0 { panic!("Assembling") }
    if !ships.contains_key(&target) {
        return assign_attack(ship, ships, strategy, grid, queue);
    } else {
        queue.push(&navigate(grid, ship, &ships[&target]));
    }
}

fn do_follow(ship: &Ship,
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
