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
            let allies = self.grid.near_allies(&ship, 1000.0, &self.ships);
            panic!("{}", allies.len())
        }
    }
}
