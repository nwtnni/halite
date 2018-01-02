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
        let player = &self.players[self.id];
        for id in &player.ships {
            let ship = &self.ships[id];
            let enemies = self.grid.near_enemies(&ship, 200.0, &self.ships);
            match enemies.get(0) {
                None => continue, 
                Some(enemy) => self.queue.push(&navigate(&mut self.grid, ship, enemy)),
            };
        }
    }
}
