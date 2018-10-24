use fnv::FnvHashSet;
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
}

impl Executor {

    pub fn new(total: Halite) -> Self {
        Executor {
            total,
            crashing: FnvHashSet::default(),
            returning: FnvHashSet::default(),
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
        );

        info!("{}", state.round);

        let yard = state.yards[state.id as usize];
        let remaining = state.halite.iter().sum::<Halite>();

        let mut allies = state.allies().collect::<Vec<_>>();
        allies.sort_by_key(|ship| constants.MAX_ENERGY - ship.halite as usize);

        let mut incoming = Vec::new();
        let mut outgoing = Vec::new();

        for ship in &allies {
            if grid.distance_from_yard(ship) as Time + state.round + 10 >= constants.MAX_TURNS as Time {
                self.crashing.insert(ship.id);
                incoming.push(ship);
            } else if ship.x == yard.x && ship.y == yard.y {
                self.returning.remove(&ship.id);
                outgoing.push(ship);
            } else if ship.halite >= 1000 {
                self.returning.insert(ship.id);
                incoming.push(ship);
            } else if self.returning.contains(&ship.id) {
                incoming.push(ship);
            } else {
                outgoing.push(ship);
            }
        }

        let mut costs = Vec::with_capacity(
            outgoing.len()
            * state.width as usize
            * state.height as usize
        );

        for ship in &outgoing {
            grid.fill_cost(&mut costs, |grid, pos, halite| {
                let cost = (constants.MAX_CELL_PRODUCTION as Halite - Halite::min(halite, constants.MAX_CELL_PRODUCTION as Halite)) / 200
                         + grid.dist(pos, Pos(yard.x, yard.y)) as Halite
                         + grid.dist(Pos(ship.x, ship.y), pos) as Halite;

                if pos == Pos(yard.x, yard.y) {
                    Halite::max_value()
                } else if halite >= 100 && !grid.is_stuck(pos) && !(grid.enemies_around(pos, 2) > 0)  {
                    cost
                } else if halite >= 12 && halite < 100 {
                    cost + 100000
                } else {
                    Halite::max_value()
                }
            });
        }

        let assignment = minimize(&costs, outgoing.len(), state.width as usize * state.height as usize)
            .into_iter()
            .enumerate();

        for (id, dest) in assignment {
            if let Some(dest) = dest {
                let ship = outgoing[id];
                let dest = Pos(dest as Dist % state.width, dest as Dist / state.width);
                grid.plan_route(ship, dest, false);
            }
        }

        for ship in incoming {
            if self.crashing.contains(&ship.id) {
                grid.plan_route(ship, Pos(yard.x, yard.y), true);
            } else if self.returning.contains(&ship.id) {
                let crowd = grid.allies_around(Pos(yard.x, yard.y), 1);
                let distance = grid.distance_from_yard(ship);

                if crowd >= 6 && distance <= 5 && distance >= 2 {
                    grid.plan_route(ship, Pos(ship.x, ship.y), false);
                } else {
                    grid.plan_route(ship, Pos(yard.x, yard.y), false);
                }
            }
        }

        let (spawnable, mut commands) = grid.resolve_routes();

        if state.halite() >= constants.NEW_ENTITY_ENERGY_COST as Halite
        && remaining as Halite >= self.total / 2
        && state.round <= (constants.MAX_TURNS / 2) as Time
        && spawnable {
            commands.push(Command::Spawn);
        }

        commands
    }
}
