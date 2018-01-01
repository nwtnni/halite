use hlt::state::*;
use hlt::strategy::*;

pub trait General {
    fn run(self);
}

impl General for State {
    fn run(self) {

    }
}
