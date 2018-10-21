use fnv::FnvHashSet;
use hungarian::minimize;

use constants::*;
use command::Command;
use data::State;
use grid::{Pos, Grid};

#[derive(Default, Debug, Clone)]
pub struct Executor {
    returning: FnvHashSet<usize>,
    routes: Vec<Vec<(usize, usize)>>,
}

impl Executor {

    pub fn execute(&mut self, state: &State) -> Vec<Command> {

        let mut grid = Grid::new(
            state.id,
            state.width,
            state.height,
            state.round,
            &state.halite,
            &state.ships,
            &state.drops,
            &state.yards,
            &mut self.routes,
        );

        let mut commands = Vec::new();
        let num_allies = state.allies().count();

        if state.halite() > NEW_ENTITY_ENERGY_COST && state.round < MAX_TURNS / 2  && grid.can_spawn() {
            grid.mark_spawn();
            commands.push(Command::Spawn);
        }

        let mut costs = Vec::with_capacity(num_allies * state.width * state.height);
        let allies = state.allies().collect::<Vec<_>>();
        let yard = state.yards[state.id];

        for ally in &allies {
            grid.fill_cost(&mut costs, |pos, halite, _surround, _allies, _enemies| {
                if halite > 100 {
                    grid.dist(pos, Pos(yard.x, yard.y)) + grid.dist(Pos(ally.x, ally.y), pos)
                } else {
                    usize::max_value()
                }
            });
        }

        let assignment = minimize(&costs, num_allies, state.width * state.height);

        for (id, dest) in assignment.into_iter().enumerate() {
            let ship = allies[id];

            if ship.halite >= RETURN || grid.distance_from_yard(ship) + state.round == MAX_TURNS {
                self.returning.insert(ship.id);
            } else if ship.x == yard.x && ship.y == yard.y {
                self.returning.remove(&ship.id);
            }

            if self.returning.contains(&ship.id) {
                let comm = grid.navigate(ship, Pos(yard.x, yard.y));
                commands.push(comm); 
            } else if let Some(dest) = dest {
                let dest = Pos(dest % state.width, dest / state.width);
                let comm = grid.navigate(ship, dest);
                commands.push(comm);
            }
        }

        commands
    }
}
