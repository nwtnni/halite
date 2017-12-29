use fnv::FnvBuildHasher;
use std::io::stdin;
use std::collections::HashMap;
use state::*;
use parse::*;
use collision::HashGrid;
use constants::BOT_NAME;

pub enum Command {
    Dock(usize, usize),
    Undock(usize),
    Thrust(usize, i32, i32),
}

pub struct CommandQueue {
    commands: String,
}

impl CommandQueue {
    pub fn new() -> Self {
        CommandQueue { commands: String::new() }
    }

    pub fn push(&mut self, command: &Command) {
        use self::Command::*;
        let string = match *command {
            Dock(ship, planet) => format!("d {} {} ", ship, planet),
            Undock(ship) => format!("u {} ", ship),
            Thrust(ship, m, a) => format!("t {} {} {} ", ship, m, a),
        };
        self.commands.push_str(&string);
    }

    pub fn flush(&mut self) {
        println!("{}", self.commands);
        self.commands.clear();
    }
}

#[derive(Debug)]
pub struct Game {
    pub id: ID,
    pub width: f32,
    pub height: f32,
    pub players: Vec<Player>,
    pub planets: HashMap<ID, Planet, FnvBuildHasher>,
    pub ships: HashMap<ID, Ship, FnvBuildHasher>,
    pub grid: HashGrid,
}

impl Game {
    pub fn new() -> Self {
        let mut buffer = String::new();
        stdin().read_line(&mut buffer).unwrap();
        stdin().read_line(&mut buffer).unwrap();
        stdin().read_line(&mut buffer).unwrap();
        let mut stream: Vec<_> = buffer.split_whitespace().rev().collect();
        let id = usize::take(&mut stream);
        let width = f32::take(&mut stream);
        let height = f32::take(&mut stream);
        let (players, planets, ships, grid) = take(&mut stream);
        Game { id, width, height, players, planets, ships, grid }
    }

    pub fn update(&mut self) {
        let mut buffer = String::new();
        stdin().read_line(&mut buffer).unwrap();
        let mut stream: Vec<_> = buffer.split_whitespace().rev().collect();
        let (players, planets, ships, grid) = take(&mut stream);
        self.players = players;
        self.planets = planets;
        self.ships = ships;
        self.grid = grid;
    }

    pub fn send_ready() {
        println!("{}", BOT_NAME);
    }
}
