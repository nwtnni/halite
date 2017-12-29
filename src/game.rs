use std::io::{stdin};
use state::*;
use parse::*;
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

pub struct Game {
    pub id: ID,
    pub width: f32,
    pub height: f32,
    pub map: Map,
}


impl Game {
    pub fn new() -> Self {
        let mut buffer = String::new();
        stdin().read_line(&mut buffer).unwrap();
        stdin().read_line(&mut buffer).unwrap();
        stdin().read_line(&mut buffer).unwrap();
        let mut stream: Vec<_> = buffer.split_whitespace().rev().collect();
        Game {
            id: usize::take(&mut stream),
            width: f32::take(&mut stream),
            height: f32::take(&mut stream),
            map: Map::take(&mut stream),
        }
    }

    pub fn update(&mut self) {
        let mut buffer = String::new();
        stdin().read_line(&mut buffer).unwrap();
        let mut stream: Vec<_> = buffer.split_whitespace().rev().collect();
        self.map = Map::take(&mut stream);
    }

    pub fn send_ready() {
        println!("{}", BOT_NAME);
    }
}
