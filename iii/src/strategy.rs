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

        let num_allies = state.allies().count();
        let yard = state.yards[state.id];

        let mut costs = Vec::with_capacity(num_allies * state.width * state.height);
        let mut allies = state.allies().collect::<Vec<_>>();
        allies.sort_by_key(|ship| ship.halite);

        for ship in &allies {
            if ship.x == yard.x && ship.y == yard.y {
                info!("Round {}: {:?}", state.round, ship);
            }
        }

        let cutoff = if grid.average_halite() > 50 { 50 } else { 25 };

        for ally in &allies {
            grid.fill_cost(&mut costs, constants.INSPIRATION_RADIUS, |grid, pos, halite| {
                if halite > cutoff && !grid.is_stuck(pos) {
                    grid.dist(pos, Pos(yard.x, yard.y)) + grid.dist(Pos(ally.x, ally.y), pos)
                } else {
                    usize::max_value()
                }
            });
        }

        let assignment = minimize(&costs, num_allies, state.width * state.height)
            .into_iter()
            .enumerate();

        for (id, dest) in assignment {
            let ship = allies[id];

            if grid.distance_from_yard(ship) + state.round + 5 >= constants.MAX_TURNS {
                self.crashing.insert(ship.id);
            } else if ship.x == yard.x && ship.y == yard.y {
                self.returning.remove(&ship.id);
            } else if ship.halite >= RETURN {
                self.returning.insert(ship.id);
            }

            if self.crashing.contains(&ship.id) {
                grid.plan_route(ship, Pos(yard.x, yard.y));
            } else if self.returning.contains(&ship.id) {
                grid.plan_route(ship, Pos(yard.x, yard.y));
            } else if let Some(dest) = dest {
                let dest = Pos(dest % state.width, dest / state.width);
                grid.plan_route(ship, dest);
            }
        }

        let (spawnable, mut commands) = grid.resolve_routes();

        if state.halite() >= constants.NEW_ENTITY_ENERGY_COST && state.round < constants.MAX_TURNS / 2  && spawnable {
            commands.push(Command::Spawn);
        }

        commands
    }
}
