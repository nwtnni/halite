use hlt::collision::*;
use hlt::command::*;
use hlt::state::*;
use hlt::strategy::*;

pub trait General {
    fn run(self);
    fn step(&mut self);
}

impl General for State {
    fn run(mut self) {
        loop {
            self.update();
            self.step();
            self.queue.flush();
        }
    }

    fn step(&mut self) {
        info!("New turn");
        let player = &mut self.players[self.id];
        for id in &player.ships {
            info!("Find a planet");
            let ship = &self.ships[id];
            let planet = &self.planets.values()
            .filter(|&planet| planet.has_spots() && !planet.is_enemy(ship.owner))
            .min_by_key(|planet| {
                (ship.distance_to(planet) - planet.value()) as i32
            }).expect("No planets left");
            if ship.in_docking_range(planet) {
                self.queue.push(&dock(ship, planet));
            } else {
                self.queue.push(
                    &navigate(&mut self.grid, ship, planet)
                );
            }
        }
    }
}
