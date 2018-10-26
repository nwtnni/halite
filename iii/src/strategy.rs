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
    reserved: FnvHashMap<(Pos, Time), ID>,
    routes: FnvHashMap<ID, VecDeque<(Pos, Time)>>,
}

impl Executor {

    pub fn new(total: Halite) -> Self {
        Executor {
            total,
            crashing: FnvHashSet::default(),
            returning: FnvHashSet::default(),
            reserved: FnvHashMap::default(),
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
        let ships = state.ships.iter()
            .filter(|ship| ship.owner == state.id)
            .cloned()
            .collect::<Vec<_>>();

        let mut costs = Vec::with_capacity(
            ships.len() * (state.width as usize) * (state.height as usize)
        );

        let mut repath = Vec::new();
        let mut cached = Vec::new();

        for (idx, ship) in ships.iter().enumerate() {
            // Returning/crashing logic
            if grid.dist_from_yard(ship) as Time + state.round + 10 >= constants.MAX_TURNS as Time {
                self.crashing.insert(ship.id);
                grid.clear_route(ship.id); 
            } else if ship.halite >= 950 {
                self.returning.insert(ship.id);
                grid.clear_route(ship.id); 
            } else if Pos(ship.x, ship.y) == Pos(yard.x, yard.y) {
                self.returning.remove(&ship.id);
            }

            // Path ordering logic
            if grid.has_cached_route(ship.id) {
                cached.push((idx, ship));
            } else {
                repath.push((idx, ship));
            }
        }

        for ship in &ships {
            if self.returning.contains(&ship.id) || self.crashing.contains(&ship.id) {
                grid.fill_cost(&mut costs, |_, pos, _| {
                    if pos == Pos(yard.x, yard.y) {
                        0
                    } else {
                        Halite::max_value()
                    }
                })
            } else {
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
        }

        let assignment = minimize(&costs, ships.len(), state.width as usize * state.height as usize)
            .into_iter()
            .map(|dest| dest.expect("[INTERNAL ERROR]: all ships should have assignment"))
            .map(|dest| grid.inv_idx(dest))
            .collect::<Vec<_>>();

        let mut commands = Vec::with_capacity(ships.len());

        for (idx, ship) in repath.iter().chain(&cached) {
            if self.crashing.contains(&ship.id) {
                commands.push(grid.navigate(ship, Pos(yard.x, yard.y), Time::max_value(), true));
            } else if self.returning.contains(&ship.id) {
                // info!("{}: repathing ship {} to {:?}", state.round, ship.id, Pos(yard.x, yard.y));
                commands.push(grid.navigate(ship, Pos(yard.x, yard.y), Time::max_value(), false));
            } else {
                // info!("{}: repathing ship {} to {:?}", state.round, ship.id, assignment[index]);
                let dist = grid.dist(Pos(ship.x, ship.y), assignment[*idx]) as Time;
                if dist <= 5 {
                    commands.push(grid.navigate(ship, assignment[*idx], 1, false));
                } else {
                    commands.push(grid.navigate(ship, assignment[*idx], dist - 5, false));
                }
            }
        }

        if grid.can_spawn()
        && state.halite.iter().sum::<Halite>() * 2 > self.total
        && state.scores[state.id as usize] >= constants.NEW_ENTITY_ENERGY_COST as Halite
        && state.round <= (constants.MAX_TURNS / 2) as Time {
            commands.push(Command::Spawn)
        }

        commands
    }
}
