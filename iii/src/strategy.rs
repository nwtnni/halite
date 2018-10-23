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

        info!("{}", state.round);

        let yard = state.yards[state.id];

        let mut allies = state.allies().collect::<Vec<_>>();
        allies.sort_by_key(|ship| ship.halite);

        let mut incoming = Vec::new();
        let mut outgoing = Vec::new();

        for ship in &allies {
            if grid.distance_from_yard(ship) + state.round + 10 >= constants.MAX_TURNS {
                self.crashing.insert(ship.id);
                incoming.push(ship);
            } else if ship.x == yard.x && ship.y == yard.y {
                self.returning.remove(&ship.id);
                outgoing.push(ship);
            } else if ship.halite >= RETURN {
                self.returning.insert(ship.id);
                incoming.push(ship);
            } else if self.returning.contains(&ship.id) {
                incoming.push(ship);
            } else {
                outgoing.push(ship);
            }
        }

        let mut costs = Vec::with_capacity(outgoing.len() * state.width * state.height);
        for ship in &outgoing {
            grid.fill_cost(&mut costs, |grid, pos, halite| {
                if halite > 50 && !grid.is_stuck(pos) {
                    grid.dist(pos, Pos(yard.x, yard.y)) + grid.dist(Pos(ship.x, ship.y), pos)
                } else {
                    usize::max_value()
                }
            });
        }

        let assignment = minimize(&costs, outgoing.len(), state.width * state.height)
            .into_iter()
            .enumerate();

        for (id, dest) in assignment {
            if let Some(dest) = dest {
                let ship = outgoing[id];
                let dest = Pos(dest % state.width, dest / state.width);
                grid.plan_route(ship, dest, false);
            }
        }

        for ship in incoming {
            if self.crashing.contains(&ship.id) {
                grid.plan_route(ship, Pos(yard.x, yard.y), true);
            } else if self.returning.contains(&ship.id) {
                grid.plan_route(ship, Pos(yard.x, yard.y), false);
            }
        }

        let (spawnable, mut commands) = grid.resolve_routes();

        if state.halite() >= constants.NEW_ENTITY_ENERGY_COST && state.round < constants.MAX_TURNS / 2  && spawnable {
            commands.push(Command::Spawn);
        }

        commands
    }
}
