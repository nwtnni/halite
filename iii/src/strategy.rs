use std::collections::VecDeque;

use indexmap::IndexSet;
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

        // Returning and crashing logic
        for ship in &ships {
            if grid.dist_from_yard(ship) as Time + state.round + 10 >= constants.MAX_TURNS as Time {
                self.crashing.insert(ship.id);
                grid.clear_route(ship.id); 
            } else if ship.halite > 950 {
                self.returning.insert(ship.id);
                grid.clear_route(ship.id); 
            } else if Pos(ship.x, ship.y) == Pos(yard.x, yard.y) {
                self.returning.remove(&ship.id);
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
                    } else if halite >= 100 && grid.enemies_around(pos, 2) == 0 {
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

        let mut commands = vec![Command::Spawn; ships.len()];
        let mut repath = ships.iter()
            .enumerate()
            .collect::<IndexSet<_>>();

        while let Some((idx, ship)) = repath.pop() {

            let crash = self.crashing.contains(&ship.id);
            let destination = if self.crashing.contains(&ship.id) || self.returning.contains(&ship.id) {
                Pos(yard.x, yard.y)
            } else {
                assignment[idx]
            };

            let depth = if self.crashing.contains(&ship.id) || self.returning.contains(&ship.id) {
                Time::max_value()
            } else {
                let dist = grid.dist(ship.into(), destination) as Time;
                if dist <= 5 { 1 } else { dist - 5 }
            };

            let (invalidated, command) = grid.navigate(ship, destination, depth, crash);

            if let Some(id) = invalidated {
                warn!("Ship {}'s route invalidated", id);
                grid.clear_route(id);
                repath.insert( 
                    ships.iter()
                        .enumerate()
                        .find(|(_, ship)| id == ship.id)
                        .expect("[INTERNAL ERROR]: missing ship")
                );
            }
            
            commands[idx] = command;
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
