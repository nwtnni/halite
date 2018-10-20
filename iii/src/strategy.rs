use fnv::FnvHashSet;
use hungarian::minimize;

use constants::*;
use command::Command;
use data::State;
use grid::{Pos, Grid};

#[derive(Default, Debug, Clone)]
pub struct Executor {
    returning: FnvHashSet<usize>,
}

impl Executor {

    pub fn execute(&mut self, state: &State) -> Vec<Command> {

        let mut grid = Grid::new(
            state.id,
            state.width,
            state.height,
            &state.halite,
            &state.ships,
            &state.drops,
            &state.yards,
        );

        let mut commands = Vec::new();
        let num_allies = state.allies().count();

        if state.halite() > NEW_ENTITY_ENERGY_COST && state.round < MAX_TURNS / 2  && grid.can_spawn() {
            grid.spawn();
            commands.push(Command::Spawn);
        }

        let mut costs = Vec::with_capacity(num_allies * state.width * state.height);
        let allies = state.allies().collect::<Vec<_>>();
        for ship in &allies { grid.distances_from(Pos(ship.x, ship.y), 50, &mut costs); }
        let assignment = minimize(&costs, num_allies, state.width * state.height);
        let yard = state.yards[state.id];

        for (id, dest) in assignment.into_iter().enumerate() {
            let ship = allies[id];

            if ship.halite >= RETURN {
                self.returning.insert(ship.id);
            } else if ship.x == yard.x && ship.y == yard.y {
                self.returning.remove(&ship.id);
            }

            if self.returning.contains(&ship.id) {
                let comm = grid.navigate(*ship, Pos(yard.x, yard.y), true);
                commands.push(comm); 
            } else if let Some(dest) = dest {
                let dest = Pos(dest % state.width, dest / state.width);
                let comm = grid.navigate(*ship, dest, false);
                commands.push(comm);
            }
        }

        commands
    }
}
