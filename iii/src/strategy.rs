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

        for ally in &allies {
            grid.fill_cost(&mut costs, |grid, pos, halite| {
                if halite > 40 {
                    grid.dist(Pos(yard.x, yard.y), pos) as Halite
                    + grid.dist(ally.into(), pos) as Halite
                } else {
                    Halite::max_value()
                }
            });

            if ally.halite > 800 {
                self.returning.insert(ally.id);
                grid.clear_route(ally.id); 
            } else if Pos(ally.x, ally.y) == Pos(yard.x, yard.y) {
                self.returning.remove(&ally.id);
            }
        }

        let assignment = minimize(&costs, allies.len(), state.width as usize * state.height as usize)
            .into_iter()
            .map(|dest| dest.expect("[INTERNAL ERROR]: all ships should have assignment"))
            .map(|dest| grid.inv_idx(dest))
            .collect::<Vec<_>>();

        let mut commands = Vec::with_capacity(allies.len());
        let repath = grid.execute_routes(&allies, &mut commands);

        for id in repath {
            let (index, ship) = allies.iter()
                .enumerate()
                .find(|(_, ship)| ship.id == id)
                .expect("[INTERNAL ERROR]: missing repathing ship");

            if self.returning.contains(&id) {
                info!("{}: repathing ship {} to {:?}", state.round, ship.id, Pos(yard.x, yard.y));
                commands.push(grid.plan_route(ship, Pos(yard.x, yard.y)));
            } else {
                info!("{}: repathing ship {} to {:?}", state.round, ship.id, assignment[index]);
                commands.push(grid.plan_route(ship, assignment[index]));
            }
        }

        if grid.can_spawn()
        && state.scores[state.id as usize] as usize >= constants.NEW_ENTITY_ENERGY_COST
        && state.round <= (constants.MAX_TURNS / 2) as Time {
            commands.push(Command::Spawn)
        }

        commands
    }
}
