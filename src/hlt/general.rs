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
        let player = &self.players[self.id];
        for id in &player.ships {
            info!("Find a planet");
            let ship = &self.ships[id];
            let to_dock = &self.planets.values()
                .filter(|&planet| {
                    planet.has_spots() 
                    && !planet.is_enemy(ship.owner)
                })
                .min_by_key(|planet| {
                    self.plan.docking_at(planet.id, &self.planets).pow(2) +
                    (ship.distance_to(planet) - planet.value()) as i32
                });

            let to_attack = &self.planets.values()
                .filter(|planet| planet.is_enemy(ship.owner))
                .min_by_key(|planet| {
                    ship.distance_to(planet) as i32 +
                    self.grid.near_enemies(planet, 35.0, &self.ships).len().pow(2) as i32
                });

            match *to_dock {
                Some(planet) => {
                    self.plan.set(ship.id, Tactic::Dock(planet.id));
                    if ship.in_docking_range(planet) {
                        self.queue.push(&dock(ship, planet));
                    } else {
                        self.queue.push(
                            &navigate(&mut self.grid, ship, &planet)
                        );
                    }
                },
                None => {
                    let to_attack = to_attack.unwrap();
                    let enemies = self.grid.near_enemies(&to_attack, 35.0, &self.ships);
                    match enemies.iter().find(|&ship| ship.is_docked()) {
                        Some(target) => {
                            self.queue.push(
                                &navigate(&mut self.grid, ship, target)
                            );
                        },
                        None => {
                            self.queue.push(
                                &navigate(&mut self.grid, &ship, &to_attack),
                            );
                        },
                    }
                }
            }
        }
    }
}
