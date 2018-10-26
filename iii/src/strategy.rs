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

        let yard = state.yards[state.id as usize];
        let allies = state.ships.iter()
            .filter(|ship| ship.owner == state.id)
            .cloned()
            .collect::<Vec<_>>();

        let mut costs = Vec::with_capacity(
            allies.len() * (state.width as usize) * (state.height as usize)
        );

        let mut incoming = Vec::new();
        let mut outgoing = Vec::new();

        for ally in &allies {
            if grid.dist_from_yard(ally) as Time + state.round + 10 >= constants.MAX_TURNS as Time {
                self.crashing.insert(ally.id);
                grid.clear_route(ally.id); 
                incoming.push(ally);
            } else if ally.halite >= 950 {
                self.returning.insert(ally.id);
                incoming.push(ally);
                grid.clear_route(ally.id); 
            } else if Pos(ally.x, ally.y) == Pos(yard.x, yard.y) {
                self.returning.remove(&ally.id);
                outgoing.push(ally);
            } else if self.returning.contains(&ally.id) || self.crashing.contains(&ally.id) {
                incoming.push(ally);
            } else {
                outgoing.push(ally);
            }
        }

        for ship in &outgoing {
            grid.fill_cost(&mut costs, |grid, pos, halite| {
                let cost = (constants.MAX_CELL_PRODUCTION as Halite -
                            Halite::min(halite, constants.MAX_CELL_PRODUCTION as Halite)
                         ) / 200
                         + grid.dist(pos, Pos(yard.x, yard.y)) as Halite
                         + grid.dist(Pos(ship.x, ship.y), pos) as Halite;
                if pos == Pos(yard.x, yard.y) {
                    Halite::max_value()
                } else if halite >= 100 {
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
            .map(|dest| dest.expect("[INTERNAL ERROR]: all ships should have assignment"))
            .map(|dest| grid.inv_idx(dest))
            .inspect(|dest| assert!(*dest != Pos(yard.x, yard.y)))
            .collect::<Vec<_>>();

        let mut commands = Vec::with_capacity(allies.len());
        let repath = grid.execute_routes(&allies, &mut commands);

        for id in repath {
            if self.crashing.contains(&id) {
                let (_, ship) = incoming.iter()
                    .enumerate()
                    .find(|(_, ship)| ship.id == id)
                    .expect("[INTERNAL ERROR]: missing repathing ship");
                commands.push(grid.plan_route(ship, Pos(yard.x, yard.y), Time::max_value(), true));
            } else if self.returning.contains(&id) {
                let (_, ship) = incoming.iter()
                    .enumerate()
                    .find(|(_, ship)| ship.id == id)
                    .expect("[INTERNAL ERROR]: missing repathing ship");
                // info!("{}: repathing ship {} to {:?}", state.round, ship.id, Pos(yard.x, yard.y));
                commands.push(grid.plan_route(ship, Pos(yard.x, yard.y), Time::max_value(), false));
            } else {
                let (index, ship) = outgoing.iter()
                    .enumerate()
                    .find(|(_, ship)| ship.id == id)
                    .expect("[INTERNAL ERROR]: missing repathing ship");
                // info!("{}: repathing ship {} to {:?}", state.round, ship.id, assignment[index]);
                let dist = grid.dist(Pos(ship.x, ship.y), assignment[index]) as Time;
                if dist <= 5 {
                    commands.push(grid.plan_route(ship, assignment[index], 1, false));
                } else {
                    commands.push(grid.plan_route(ship, assignment[index], dist - 5, false));
                }
            }
        }

        if grid.can_spawn()
        && state.halite.iter().sum::<Halite>() * 2 > self.total
        && state.scores[state.id as usize] as usize >= constants.NEW_ENTITY_ENERGY_COST
        && state.round <= (constants.MAX_TURNS / 2) as Time {
            commands.push(Command::Spawn)
        }

        commands
    }
}
