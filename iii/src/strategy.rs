use fnv::FnvHashSet;
use hungarian::minimize;

use constants::{Constants, RETURN};
use command::Command;
use data::State;
use grid::{Dir, Pos, Grid};

#[derive(Default, Debug, Clone)]
pub struct Executor {
    crashing: FnvHashSet<usize>,
    returning: FnvHashSet<usize>,
}

impl Executor {

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
        );

        let mut commands = Vec::new();
        let num_allies = state.allies().count();

        if state.halite() >= constants.NEW_ENTITY_ENERGY_COST && state.round < constants.MAX_TURNS / 2  && grid.can_spawn() {
            grid.mark_spawn();
            commands.push(Command::Spawn);
        }

        let mut costs = Vec::with_capacity(num_allies * state.width * state.height);
        let allies = state.allies().collect::<Vec<_>>();
        let yard = state.yards[state.id];
        let cutoff = if grid.average_halite() > 100 { 100 } else { 50 };

        for ally in &allies {
            grid.fill_cost(&mut costs, constants.INSPIRATION_RADIUS, |pos, halite, _surround, _allies, _enemies| {
                if halite > cutoff {
                    grid.dist(pos, Pos(yard.x, yard.y)) + grid.dist(Pos(ally.x, ally.y), pos)
                } else {
                    usize::max_value()
                }
            });
        }

        let assignment = minimize(&costs, num_allies, state.width * state.height);

        let mut sorted = assignment.into_iter()
            .enumerate()
            .collect::<Vec<_>>();

        sorted.sort_by_key(|(id, _)| allies[*id].halite);

        for (id, dest) in sorted {
            let ship = allies[id];

            if grid.distance_from_yard(ship) + state.round + 5 >= constants.MAX_TURNS {
                self.crashing.insert(ship.id);
            } else if ship.x == yard.x && ship.y == yard.y {
                self.returning.remove(&ship.id);
            } else if ship.halite >= RETURN { 
                self.returning.insert(ship.id);
            }

            let comm = if self.crashing.contains(&ship.id) {
                grid.navigate(ship, Pos(yard.x, yard.y), true)
            } else if self.returning.contains(&ship.id) {
                grid.navigate(ship, Pos(yard.x, yard.y), false)
            } else if let Some(dest) = dest {
                let dest = Pos(dest % state.width, dest / state.width);
                grid.navigate(ship, dest, false)
            } else {
                Command::Move(ship.id, Dir::O)    
            };

            commands.push(comm);
        }

        commands
    }
}
