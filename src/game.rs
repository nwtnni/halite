use std::io::{stdin, stdout, Write};
use state::*;
use parse::*;
use constants::BOT_NAME;

pub enum Command {
    Dock(usize, usize),
    Undock(usize),
    Thrust(usize, i32, i32),
}

pub struct Game {
    pub id: ID,
    pub width: f32,
    pub height: f32,
    pub map: Map,
    commands: String,
}

impl Game {
    pub fn new() -> Self {
        let mut buffer = String::new();
        stdin().read_line(&mut buffer).unwrap();
        stdin().read_line(&mut buffer).unwrap();
        stdin().read_line(&mut buffer).unwrap();
        let mut stream: Vec<_> = buffer.split_whitespace().collect();
        Game {
            id: usize::take(&mut stream),
            width: f32::take(&mut stream),
            height: f32::take(&mut stream),
            map: Map::take(&mut stream),
            commands: String::new(),
        }
    }

    pub fn update(&mut self) {
        let mut buffer = String::new();
        stdin().read_line(&mut buffer).unwrap();
        let mut stream: Vec<_> = buffer.split_whitespace().collect();
        self.map = Map::take(&mut stream);
    }

    pub fn send_ready() {
        println!("{}", BOT_NAME);
    }

    pub fn push_queue(&mut self, command: &Command) {
        use self::Command::*;
        let string = match *command {
            Dock(ship, planet) => format!("d {} {}\n", ship, planet),
            Undock(ship) => format!("u {}\n", ship),
            Thrust(ship, m, a) => format!("t {} {} {}\n", ship, m, a),
        };
        self.commands.push_str(&string);
    }

    pub fn flush_queue(&mut self) {
        print!("{}", self.commands);
        stdout().flush().unwrap();
        self.commands.clear();
    }
}
