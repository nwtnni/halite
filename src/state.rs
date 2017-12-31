use fnv::FnvHashMap;
use std::io::stdin;
use parse::*;
use collision::*;

pub type ID = usize;
pub type Point = (f32, f32);
pub type Planets = FnvHashMap<ID, Planet>;
pub type Ships = FnvHashMap<ID, Ship>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Docking, Docked, Undocked, Undocking
}

#[derive(Debug)]
pub struct Player {
    pub id: ID,
    pub ships: Vec<ID>,
}

#[derive(Debug)]
pub struct Ship {
    pub id: ID,
    pub x: f32,
    pub y: f32,
    pub hp: i32,
    pub rad: f32,
    pub status: Status,
    pub planet: Option<ID>,
    pub progress: i32,
}

#[derive(Debug)]
pub struct Planet {
    pub id: ID,
    pub x: f32,
    pub y: f32,
    pub hp: i32,
    pub rad: f32,
    pub spots: i32,
    pub spawn: i32,
    pub owner: Option<ID>,
    pub ships: Vec<usize>,
}

#[derive(Debug)]
pub struct Game {
    pub id: ID,
    pub width: f32,
    pub height: f32,
    pub players: Vec<Player>,
    pub planets: Planets,
    pub ships: Ships,
    pub grid: Grid,
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

    pub fn send_ready(name: &str) {
        println!("{}", name);
    }
}
