use constants::*;
use command::Command;
use data::State;

pub fn execute(state: &State) -> Vec<Command> {

    let mut commands = Vec::new();

    if state.halite() > NEW_ENTITY_ENERGY_COST && state.round < MAX_TURNS / 2 {
        commands.push(Command::Spawn);
    }

    for ship in state.allies() {

    }

    commands
}
