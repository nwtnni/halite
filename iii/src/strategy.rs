use hungarian::minimize;

use constants::*;
use command::Command;
use data::State;
use grid::{Pos, Grid};

pub fn execute(state: &State) -> Vec<Command> {

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
    for ship in &allies { grid.distances_from(Pos(ship.x, ship.y), 20, &mut costs); }
    let assignment = minimize(&costs, num_allies, state.width * state.height);
    let yard = state.yards[state.id];

    for (id, dest) in assignment.into_iter().enumerate() {
        let ship = allies[id];
        if ship.halite > 50 {
            let comm = grid.navigate(*ship, Pos(yard.x, yard.y));
            commands.push(comm); 
        } else if let Some(dest) = dest {
            let dest = Pos(dest % state.width, dest / state.width);
            let comm = grid.navigate(*ship, dest);
            commands.push(comm);
        } else {
            commands.push(Command::Stay(id));
        }
    }

    commands
}
