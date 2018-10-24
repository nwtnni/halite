use std::collections::VecDeque;

use fnv::{FnvHashMap, FnvHashSet};
use hungarian::minimize;

use constants::Constants;
use command::Command;
use data::*;
use grid::Grid;

#[derive(Debug, Clone)]
pub struct Executor {
    total: Halite,
    crashing: FnvHashSet<ID>,
    returning: FnvHashSet<ID>,
    reserved: FnvHashSet<(Pos, Time)>,
    routes: FnvHashMap<ID, VecDeque<Pos>>,
}

impl Executor {

    pub fn new(total: Halite) -> Self {
        Executor {
            total,
            crashing: FnvHashSet::default(),
            returning: FnvHashSet::default(),
            reserved: FnvHashSet::default(),
            routes: FnvHashMap::default(),
        }
    }

    pub fn execute(&mut self, constants: &Constants, state: &State) -> Vec<Command> {

        let mut grid = Grid::new(
            state.id,   
            state.width,
            state.height,
            state.round,
            &state.halite,
            &state.ships,
            &state.drops,
            &state.yards,
            &mut self.reserved,
            &mut self.routes,
        );

        unimplemented!()

    }
}
